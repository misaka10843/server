use std::panic::catch_unwind;

use anyhow::bail;
use itertools::assert_equal;
use sea_query::Iden;
use serde_json::{from_value, json};
use thcdb_rs::application;
use thcdb_rs::application::auth::{AuthServiceTrait, SignInError, SignUpError};
use thcdb_rs::domain::model::auth::{AuthCredential, AuthnError};
use thcdb_rs::infra::database::sea_orm::SeaOrmRepository;

use crate::common::database::{TestDatabase, with_test_db};

type AuthService = application::auth::AuthService<SeaOrmRepository>;

#[tokio::test]
async fn test_user_sign_up() -> anyhow::Result<()> {
    with_test_db(|conn| async move {
        let service = AuthService::new(SeaOrmRepository::new(conn));

        let signup_data = json!({
            "username": "testuser",
            "password": "testpassword123@!"
        });
        let creds: AuthCredential = from_value(signup_data)?;
        let user = service.sign_up(creds).await?;

        assert_eq!(user.name, "testuser");
        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_user_sign_in() -> anyhow::Result<()> {
    with_test_db(|conn| async move {
        let service = AuthService::new(SeaOrmRepository::new(conn));

        let signup_data = json!({
            "username": "testuser",
            "password": "testpassword123@!"
        });
        let creds: AuthCredential = from_value(signup_data)?;
        service.sign_up(creds).await?;

        let signin_data = json!({
            "username": "testuser",
            "password": "testpassword123@!"
        });
        let creds: AuthCredential = from_value(signin_data)?;
        let user = service.sign_in(creds).await?;

        assert_eq!(user.name, "testuser");
        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_user_sign_up_duplicate_name() -> anyhow::Result<()> {
    with_test_db(|conn| async move {
        let service = AuthService::new(SeaOrmRepository::new(conn));

        let creds: AuthCredential = from_value(json!({
            "username": "testuser",
            "password": "testpassword123@!"
        }))?;
        service.sign_up(creds).await?;

        let creds_dup: AuthCredential = from_value(json!({
            "username": "testuser",
            "password": "anotherpassword@!"
        }))?;
        let result = service.sign_up(creds_dup).await;

        assert!(matches!(
            result,
            Err(SignUpError::UsernameAlreadyInUse { .. })
        ));
        Ok(())
    })
    .await
}

#[tokio::test]
async fn test_user_sign_in_invalid_credentials() -> anyhow::Result<()> {
    with_test_db(|conn| async move {
        let service = AuthService::new(SeaOrmRepository::new(conn));

        let creds: AuthCredential = from_value(json!({
            "username": "nonexistent",
            "password": "wrongpassword"
        }))?;
        let result = service.sign_in(creds).await;

        assert!(matches!(
            result,
            Err(SignInError::Authn(AuthnError::AuthenticationFailed { .. }))
        ));

        Ok(())
    })
    .await
}
