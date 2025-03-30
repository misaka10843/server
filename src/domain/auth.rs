use std::sync::LazyLock;

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{self, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::http::StatusCode;
use error_set::error_set;
use juniper::GraphQLInputObject;
use macros::ApiError;
use regex::Regex;
use serde::{Deserialize, Serialize};
use tokio::task::JoinError;
use utoipa::ToSchema;

use crate::api_response::StatusCodeExt;
use crate::error::{ApiErrorTrait, AsErrorCode, ErrorCode, TokioError};
use crate::state::ARGON2_HASHER;

error_set! {
    #[derive(ApiError)]
    #[disable(From(TokioError, HasherError))]
    AuthnError = {
        #[api_error(
            status_code = StatusCode::UNAUTHORIZED,
            error_code = ErrorCode::AuthenticationFailed,
            into_response = self
        )]
        #[display("Incorrect username or password")]
        AuthenticationFailed,
        Hash(HasherError),
        Tokio(TokioError),
    };
    ValidateCredsError = {
        #[display("Invalid username")]
        InvalidUserName,
        #[display("Invalid Password")]
        InvalidPassword,
        #[display("Password is too weak")]
        PasswordTooWeak,
    };
    HasherError = {
        #[display("Failed to hash password")]
        HashPasswordFailed {
            err: password_hash::errors::Error
        },
    };
}

impl From<JoinError> for AuthnError {
    fn from(value: JoinError) -> Self {
        Self::Tokio(value.into())
    }
}

impl From<password_hash::Error> for AuthnError {
    fn from(value: password_hash::Error) -> Self {
        Self::Hash(value.into())
    }
}

impl From<password_hash::Error> for HasherError {
    fn from(value: password_hash::Error) -> Self {
        Self::HashPasswordFailed { err: value }
    }
}

impl StatusCodeExt for HasherError {
    fn as_status_code(&self) -> StatusCode {
        StatusCode::INTERNAL_SERVER_ERROR
    }

    fn all_status_codes() -> impl Iterator<Item = StatusCode> {
        [StatusCode::INTERNAL_SERVER_ERROR].into_iter()
    }
}

impl AsErrorCode for HasherError {
    fn as_error_code(&self) -> ErrorCode {
        ErrorCode::InternalServerError
    }
}

impl ApiErrorTrait for HasherError {
    fn before_into_api_error(&self) {
        tracing::error!("Hasher error: {}", self);
    }
}

impl StatusCodeExt for ValidateCredsError {
    fn as_status_code(&self) -> StatusCode {
        StatusCode::BAD_REQUEST
    }

    fn all_status_codes() -> impl Iterator<Item = StatusCode> {
        [StatusCode::BAD_REQUEST].into_iter()
    }
}

impl AsErrorCode for ValidateCredsError {
    fn as_error_code(&self) -> ErrorCode {
        match self {
            Self::InvalidUserName => ErrorCode::InvalidUserName,
            Self::InvalidPassword => ErrorCode::InvalidPassword,
            Self::PasswordTooWeak => ErrorCode::PasswordTooWeak,
        }
    }
}

impl ApiErrorTrait for ValidateCredsError {}

#[derive(GraphQLInputObject, ToSchema, Clone, Deserialize, Serialize)]
pub struct AuthCredential {
    pub username: String,
    pub password: String,
}

impl AuthCredential {
    pub fn validate(&self) -> Result<(), ValidateCredsError> {
        validate_username(&self.username)?;
        validate_password(&self.password)?;

        Ok(())
    }

    pub fn hashed_password(
        &self,
    ) -> Result<String, password_hash::errors::Error> {
        hash_password(&self.password)
    }

    pub async fn verify_credentials(
        &self,
        hash: Option<&str>,
    ) -> Result<(), AuthnError> {
        verify_password(
            &self.hashed_password()?,
            hash.unwrap_or(&hash_password("dummyPassword")?),
        )
        .await?;

        Ok(())
    }
}

pub fn hash_password(pwd: &str) -> password_hash::Result<String> {
    let salt = SaltString::generate(&mut OsRng);

    let res = ARGON2_HASHER.hash_password(pwd.as_bytes(), &salt)?;

    Ok(res.to_string())
}

async fn verify_password(
    password: &str,
    password_hash: &str,
) -> Result<bool, AuthnError> {
    let bytes = password.as_bytes().to_owned();
    let password_hash = password_hash.to_string();

    tokio::task::spawn_blocking(move || {
        let hash = PasswordHash::new(&password_hash)?;

        Ok::<bool, AuthnError>(
            Argon2::default().verify_password(&bytes, &hash).is_ok(),
        )
    })
    .await?
}

fn validate_username(username: &str) -> Result<(), ValidateCredsError> {
    static USER_NAME_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(r"^[\p{L}\p{N}_]{1,32}$").unwrap());

    if USER_NAME_REGEX.is_match(username)
        && !username
            .chars()
            .any(|c| c.is_control() || c.is_whitespace())
    {
        Ok(())
    } else {
        Err(ValidateCredsError::InvalidUserName)
    }
}

/// Valid characters
/// - A-z
/// - 0-9
/// - \`~!@#$%^&*()-_=+
fn validate_password(password: &str) -> Result<(), ValidateCredsError> {
    use zxcvbn::{Score, zxcvbn};

    static USER_PASSWORD_REGEX: LazyLock<Regex> = LazyLock::new(|| {
        Regex::new(r"^[A-Za-z\d`~!@#$%^&*()\-_=+]{8,}$").unwrap()
    });

    if USER_PASSWORD_REGEX.is_match(password) {
        let result = zxcvbn(password, &[]);

        #[cfg(test)]
        {
            println!("password: {password}, score: {}", result.score());
        }

        match result.score() {
            Score::Three | Score::Four => Ok(()),
            _ => Err(ValidateCredsError::PasswordTooWeak),
        }
    } else {
        Err(ValidateCredsError::InvalidPassword)
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_validate_username() {
        let test_cases = [
            // 长度
            ("", false),
            (&"a".repeat(33), false),
            // 空格
            (" a ", false),
            ("a a", false),
            // 特殊字符
            ("😀", false),       // emoji
            (" ", false),        // 单个空格
            ("\n", false),       // 换行符
            ("\t", false),       // 制表符
            ("\u{200B}", false), // 零宽空格
            ("\u{00A0}", false), // 不间断空格
            ("alice_megatron", true),
            // 中文
            ("无蛋黄", true),
            ("憂鬱的臺灣烏龜", true),
            // 日文
            ("ひらがな", true),
            ("かたかな", true),
            ("カタカナ", true),
            // 韩文
            ("안녕하세요", true),
            ("사용자", true),
            // 西里尔字母
            ("пример", true),
            ("пользователь", true),
            // 德语字符
            ("müller", true),
            ("straße", true),
            // 阿拉伯字符
            ("مرحبا", true),
            ("مستخدم", true),
        ];

        for (username, expected) in test_cases {
            assert_eq!(validate_username(username).is_ok(), expected);
        }
    }

    #[test]
    fn test_validate_password() {
        let test_case = [
            ("Password123!", false),
            ("SecurePass#2023", true),
            ("HelloWorld!1", true),
            ("weak", false),
            ("password", false),
            ("PASSWORD123", false),
            ("Pass!", false),
            ("12345678", false),
            ("!@#$%^&*", false),
            ("NoSpecialChar123", true),
            ("NoNumberHere!", true),
            ("nocapitals1!", true),
            ("NOLOWERCASE1!", true),
            ("m10KSGDckKrX38Vm", true),
            ("1KrIuT%gcemHwjwF", true),
            ("a1`~!@#$%^&*()-_=+", true),
        ];

        for (password, expected) in test_case {
            println!("password: {password}, expected: {expected}");
            assert_eq!(validate_password(password).is_ok(), expected);
        }
    }
}
