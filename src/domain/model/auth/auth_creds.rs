use std::sync::LazyLock;

use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{self, SaltString};
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::http::StatusCode;
use derive_more::From;
use error_set::error_set;
use macros::ApiError;
use regex::Regex;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::constant::{USER_NAME_REGEX_STR, USER_PASSWORD_REGEX_STR};
use crate::error::{ApiErrorTrait, ErrorCode, TokioError};
use crate::state::ARGON2_HASHER;

error_set! {
    #[derive(ApiError, From)]
    #[disable(From(TokioError, HasherError))]
    AuthnError = {
        #[api_error(
            status_code = StatusCode::UNAUTHORIZED,
            error_code = ErrorCode::AuthenticationFailed,
        )]
        #[display("Incorrect username or password")]
        AuthenticationFailed,
        #[api_error(
            into_response = self
        )]
        #[from(password_hash::Error)]
        Hash(HasherError),
        #[api_error(
            into_response = self
        )]
        #[from(tokio::task::JoinError)]
        Tokio(TokioError),
    };
    #[derive(ApiError)]
    ValidateCredsError = {
        #[display("Invalid username")]
        #[api_error(
            status_code = StatusCode::BAD_REQUEST,
            error_code = ErrorCode::InvalidUserName
        )]
        InvalidUserName,
        #[display("Invalid Password")]
        #[api_error(
            status_code = StatusCode::BAD_REQUEST,
            error_code = ErrorCode::InvalidPassword
        )]
        InvalidPassword,
        #[display("Password is too weak")]
        #[api_error(
            status_code = StatusCode::BAD_REQUEST,
            error_code = ErrorCode::PasswordTooWeak
        )]
        PasswordTooWeak,
    };
    #[derive(From, ApiError)]
    #[api_error(
        impl_api_error = false
    )]
    HasherError = {
        #[display("Failed to hash password")]
        #[from]
        #[api_error(
            status_code = StatusCode::INTERNAL_SERVER_ERROR,
            error_code = ErrorCode::InternalServerError
        )]
        HashPasswordFailed {
            err: password_hash::Error
        },
    };
}

#[derive(Clone, Serialize, Deserialize, ToSchema)]
pub struct AuthCredential {
    pub username: String,
    pub password: String,
}

impl AuthCredential {
    // TODO: Validate on new
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
        let dummy_password = || hash_password("dummy_password");

        verify_password(
            hash.unwrap_or(&dummy_password()?).to_owned(),
            &self.password,
        )
        .await
    }
}

pub fn hash_password(pwd: &str) -> password_hash::Result<String> {
    let salt = SaltString::generate(&mut OsRng);

    let res = ARGON2_HASHER.hash_password(pwd.as_bytes(), &salt)?;

    Ok(res.to_string())
}

/// Return `[Err(AuthnError::AuthenticationFailed)]` if password is incorrect
/// otherwise return `Ok(())`
async fn verify_password(
    password_hash: String,
    input: &str,
) -> Result<(), AuthnError> {
    let bytes = input.as_bytes().to_owned();
    let res = tokio::task::spawn_blocking(move || {
        let hash = PasswordHash::new(&password_hash)?;

        Ok::<bool, AuthnError>(
            Argon2::default().verify_password(&bytes, &hash).is_ok(),
        )
    })
    .await??;

    if res {
        Ok(())
    } else {
        Err(AuthnError::AuthenticationFailed)
    }
}

fn validate_username(username: &str) -> Result<(), ValidateCredsError> {
    static USER_NAME_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(USER_NAME_REGEX_STR).unwrap());

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

    static USER_PASSWORD_REGEX: LazyLock<Regex> =
        LazyLock::new(|| Regex::new(USER_PASSWORD_REGEX_STR).unwrap());

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
impl ApiErrorTrait for HasherError {
    fn before_into_api_error(&self) {
        tracing::error!("Hasher error: {}", self);
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[tokio::test]
    async fn verify_password() {
        let password = "Password123123!";
        let hash = hash_password(password).unwrap();

        let res = super::verify_password(hash, password).await.is_ok();

        assert!(res);
    }

    #[tokio::test]
    async fn verify_credentials() {
        let pwd = "Password123123!".to_string();
        let res = AuthCredential {
            username: "Alice".to_string(),
            password: pwd.clone(),
        }
        .verify_credentials(Some(&hash_password(&pwd).unwrap()))
        .await
        .is_ok();

        assert!(res);
    }

    #[tokio::test]
    async fn verify_credentials_fail() {
        let pwd = "Password123123!".to_string();
        let res = AuthCredential {
            username: "Alice".to_string(),
            password: pwd.clone(),
        }
        .verify_credentials(None)
        .await
        .is_err();

        assert!(res);
    }

    #[test]
    fn test_validate_username() {
        let test_cases = [
            // 长度
            ("", false),
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
