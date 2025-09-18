use chrono::{DateTime, Utc};
use entity::enums::EntityType;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::domain::correction::CorrectionEntity;
use crate::domain::shared::model::{DateWithPrecision, EntityIdent};

#[serde_with::apply(
    Vec    => #[serde(skip_serializing_if = "Vec::is_empty")],
    Option => #[serde(skip_serializing_if = "Option::is_none")]
)]
#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct Event {
    pub id: i32,
    pub name: String,
    pub short_description: Option<String>,
    pub description: Option<String>,
    pub start_date: Option<DateWithPrecision>,
    pub end_date: Option<DateWithPrecision>,
    pub alternative_names: Vec<AlternativeName>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
#[cfg_attr(test, derive(PartialEq, Eq))]
pub struct SimpleEvent {
    pub id: i32,
    pub name: String,
}

#[derive(Clone, Debug, Serialize, ToSchema)]
pub struct AlternativeName {
    pub id: i32,
    pub name: String,
}

#[derive(Deserialize, ToSchema)]
pub struct NewEvent {
    pub name: EntityIdent,
    pub short_description: Option<String>,
    pub description: Option<String>,
    pub start_date: Option<DateWithPrecision>,
    pub end_date: Option<DateWithPrecision>,
    pub alternative_names: Option<Vec<String>>,
}

impl CorrectionEntity for NewEvent {
    fn entity_type() -> EntityType {
        EntityType::Event
    }
}
