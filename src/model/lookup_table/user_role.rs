use entity::role;
use sea_orm::ActiveValue::Set;
use sea_orm::EntityTrait;

use super::{LookupTableCheckResult, LookupTableEnum, ValidateLookupTable};
use crate::model::auth::UserRole;

impl<T> From<T> for LookupTableCheckResult<T> {
    fn from(val: T) -> Self {
        Self::Conflict(val)
    }
}

impl PartialEq<role::Model> for UserRole {
    fn eq(&self, other: &role::Model) -> bool {
        self.as_id() == other.id && self.to_string() == other.name
    }
}

pub(super) struct UserRoleConflict {
    pub id: i32,
    pub db_name: String,
    pub enum_name: String,
}

impl From<UserRole> for role::ActiveModel {
    fn from(val: UserRole) -> Self {
        Self {
            id: Set(val.as_id()),
            name: Set(val.to_string()),
        }
    }
}

#[allow(clippy::fallible_impl_from)]
impl From<&role::Model> for UserRole {
    fn from(val: &role::Model) -> Self {
        Self::try_from_id(val.id).unwrap()
    }
}

impl ValidateLookupTable for UserRole {
    type ConflictData = UserRoleConflict;

    type Entity = role::Entity;

    fn try_from_model(
        model: &<Self::Entity as EntityTrait>::Model,
    ) -> Result<Self, ()> {
        Self::try_from_id(model.id)
    }

    fn new_conflict_data(
        self,
        model: &<Self::Entity as EntityTrait>::Model,
    ) -> Self::ConflictData {
        Self::ConflictData {
            id: model.id,
            db_name: model.name.clone(),
            enum_name: self.to_string(),
        }
    }

    fn display_conflict(
        Self::ConflictData {
            id,
            db_name,
            enum_name,
        }: Self::ConflictData,
    ) -> String {
        format!(
            "User role definition conflicts with database records.\n\
            On:\n\
            - ID: {id}\n\
            - Database value: '{db_name}'\n\
            - Enum value: '{enum_name}'"
        )
    }
}

impl LookupTableEnum for UserRole {
    fn as_id(&self) -> i32 {
        match self {
            Self::Admin => 1,
            Self::Moderator => 2,
            Self::User => 3,
        }
    }

    fn try_from_id(id: i32) -> Result<Self, ()> {
        let res = match id {
            1 => Self::Admin,
            2 => Self::Moderator,
            3 => Self::User,
            _ => {
                return Err(());
            }
        };

        Ok(res)
    }
}
