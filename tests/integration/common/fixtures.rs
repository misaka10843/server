use chrono::{DateTime, Utc};
use entity::{artist, event, release, song, tag, user};
use fake::{Fake, Faker};
use sea_orm::{DatabaseConnection, EntityTrait, Set};

use super::TestError;

/// Test data factory for creating test entities
pub struct TestFixtures {
    db: DatabaseConnection,
}

impl TestFixtures {
    /// Create a new test fixtures instance
    pub fn new(db: DatabaseConnection) -> Self {
        Self { db }
    }

    /// Create a test user
    pub async fn create_user(
        &self,
        data: Option<CreateUserData>,
    ) -> Result<user::Model, TestError> {
        let data = data.unwrap_or_default();

        let user_data = user::ActiveModel {
            name: Set(data.name),
            password: Set(data.password_hash),
            avatar_id: Set(data.avatar_id),
            last_login: Set(data.last_login.into()),
            profile_banner_id: Set(data.profile_banner_id),
            bio: Set(data.bio),
            ..Default::default()
        };

        let user = user::Entity::insert(user_data)
            .exec_with_returning(&self.db)
            .await?;

        Ok(user)
    }

    /// Create a test artist
    pub async fn create_artist(
        &self,
        data: Option<CreateArtistData>,
    ) -> Result<artist::Model, TestError> {
        let data = data.unwrap_or_default();

        let artist_data = artist::ActiveModel {
            name: Set(data.name),
            artist_type: Set(data.artist_type),
            text_alias: Set(data.text_alias),
            start_date: Set(data.start_date),
            start_date_precision: Set(data.start_date_precision),
            end_date: Set(data.end_date),
            end_date_precision: Set(data.end_date_precision),
            current_location_country: Set(data.location.clone()),
            current_location_province: Set(None),
            current_location_city: Set(None),
            ..Default::default()
        };

        let artist = artist::Entity::insert(artist_data)
            .exec_with_returning(&self.db)
            .await?;

        Ok(artist)
    }

    /// Create a test song
    pub async fn create_song(
        &self,
        data: Option<CreateSongData>,
    ) -> Result<song::Model, TestError> {
        let data = data.unwrap_or_default();

        let song_data = song::ActiveModel {
            title: Set(data.title),
            ..Default::default()
        };

        let song = song::Entity::insert(song_data)
            .exec_with_returning(&self.db)
            .await?;

        Ok(song)
    }

    /// Create a test release
    pub async fn create_release(
        &self,
        data: Option<CreateReleaseData>,
    ) -> Result<release::Model, TestError> {
        let data = data.unwrap_or_default();

        let release_data = release::ActiveModel {
            title: Set(data.title),
            release_type: Set(data.release_type),
            release_date: Set(data.release_date),
            release_date_precision: Set(data.release_date_precision),
            recording_date_start: Set(data.recording_date_start),
            recording_date_start_precision: Set(
                data.recording_date_start_precision
            ),
            recording_date_end: Set(data.recording_date_end),
            recording_date_end_precision: Set(data.recording_date_end_precision),
            // Note: catalog_number field doesn't exist in the current schema
            ..Default::default()
        };

        let release = release::Entity::insert(release_data)
            .exec_with_returning(&self.db)
            .await?;

        Ok(release)
    }

    /// Create a test tag
    pub async fn create_tag(
        &self,
        data: Option<CreateTagData>,
    ) -> Result<tag::Model, TestError> {
        let data = data.unwrap_or_default();

        let tag_data = tag::ActiveModel {
            name: Set(data.name),
            r#type: Set(data.tag_type),
            short_description: Set(data.short_description),
            description: Set(data.description),
            ..Default::default()
        };

        let tag = tag::Entity::insert(tag_data)
            .exec_with_returning(&self.db)
            .await?;

        Ok(tag)
    }

    /// Create a test event
    pub async fn create_event(
        &self,
        data: Option<CreateEventData>,
    ) -> Result<event::Model, TestError> {
        let data = data.unwrap_or_default();

        let event_data = event::ActiveModel {
            name: Set(data.name),
            short_description: Set(data.short_description),
            description: Set(data.description),
            start_date: Set(data.start_date),
            start_date_precision: Set(data.start_date_precision),
            end_date: Set(data.end_date),
            end_date_precision: Set(data.end_date_precision),
            ..Default::default()
        };

        let event = event::Entity::insert(event_data)
            .exec_with_returning(&self.db)
            .await?;

        Ok(event)
    }
}

/// Data structure for creating test users
#[derive(Debug, Clone)]
pub struct CreateUserData {
    pub name: String,
    pub password_hash: String,
    pub avatar_id: Option<i32>,
    pub last_login: DateTime<Utc>,
    pub profile_banner_id: Option<i32>,
    pub bio: Option<String>,
}

impl Default for CreateUserData {
    fn default() -> Self {
        Self {
            name: format!("user_{}", Faker.fake::<u32>()),
            password_hash: "$2b$12$dummy_hash_for_testing".to_string(), /* Dummy hash for testing */
            avatar_id: None,
            last_login: Utc::now(),
            profile_banner_id: None,
            bio: Some("Test user bio".to_string()),
        }
    }
}

/// Data structure for creating test artists
#[derive(Debug, Clone)]
pub struct CreateArtistData {
    pub name: String,
    pub artist_type: entity::enums::ArtistType,
    pub text_alias: Option<Vec<String>>,
    pub start_date: Option<chrono::NaiveDate>,
    pub start_date_precision: Option<entity::enums::DatePrecision>,
    pub end_date: Option<chrono::NaiveDate>,
    pub end_date_precision: Option<entity::enums::DatePrecision>,
    pub location: Option<String>,
}

impl Default for CreateArtistData {
    fn default() -> Self {
        Self {
            name: format!("Artist {}", Faker.fake::<u32>()),
            artist_type: entity::enums::ArtistType::Solo,
            text_alias: None,
            start_date: None,
            start_date_precision: None,
            end_date: None,
            end_date_precision: None,
            location: Some("Test Location".to_string()),
        }
    }
}

/// Data structure for creating test songs
#[derive(Debug, Clone)]
pub struct CreateSongData {
    pub title: String,
}

impl Default for CreateSongData {
    fn default() -> Self {
        Self {
            title: format!("Song {}", Faker.fake::<u32>()),
        }
    }
}

/// Data structure for creating test releases
#[derive(Debug, Clone)]
pub struct CreateReleaseData {
    pub title: String,
    pub release_type: entity::enums::ReleaseType,
    pub release_date: Option<chrono::NaiveDate>,
    pub release_date_precision: entity::enums::DatePrecision,
    pub recording_date_start: Option<chrono::NaiveDate>,
    pub recording_date_start_precision: entity::enums::DatePrecision,
    pub recording_date_end: Option<chrono::NaiveDate>,
    pub recording_date_end_precision: entity::enums::DatePrecision,
    pub catalog_number: Option<String>,
}

impl Default for CreateReleaseData {
    fn default() -> Self {
        Self {
            title: format!("Release {}", Faker.fake::<u32>()),
            release_type: entity::enums::ReleaseType::Album,
            release_date: None,
            release_date_precision: entity::enums::DatePrecision::Day,
            recording_date_start: None,
            recording_date_start_precision: entity::enums::DatePrecision::Day,
            recording_date_end: None,
            recording_date_end_precision: entity::enums::DatePrecision::Day,
            catalog_number: Some(format!("CAT-{}", Faker.fake::<u32>())),
        }
    }
}

/// Data structure for creating test tags
#[derive(Debug, Clone)]
pub struct CreateTagData {
    pub name: String,
    pub tag_type: entity::enums::TagType,
    pub short_description: String,
    pub description: String,
}

impl Default for CreateTagData {
    fn default() -> Self {
        Self {
            name: format!("Tag {}", Faker.fake::<u32>()),
            tag_type: entity::enums::TagType::Genre,
            short_description: "Test tag short description".to_string(),
            description: "Test tag description".to_string(),
        }
    }
}

/// Data structure for creating test events
#[derive(Debug, Clone)]
pub struct CreateEventData {
    pub name: String,
    pub short_description: String,
    pub description: String,
    pub start_date: Option<chrono::NaiveDate>,
    pub start_date_precision: entity::enums::DatePrecision,
    pub end_date: Option<chrono::NaiveDate>,
    pub end_date_precision: entity::enums::DatePrecision,
}

impl Default for CreateEventData {
    fn default() -> Self {
        use chrono::NaiveDate;

        let start_date = NaiveDate::from_ymd_opt(2024, 1, 1);
        let end_date = NaiveDate::from_ymd_opt(2024, 1, 2);

        Self {
            name: format!("Event {}", Faker.fake::<u32>()),
            short_description: "Test event short description".to_string(),
            description: "Test event description".to_string(),
            start_date,
            start_date_precision: entity::enums::DatePrecision::Day,
            end_date,
            end_date_precision: entity::enums::DatePrecision::Day,
        }
    }
}
