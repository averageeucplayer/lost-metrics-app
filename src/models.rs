use chrono::{Date, DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum Message {
    Unknown,
    ProcessNotRunning,
    ProcessRunning,
    ProcessNotListening,
    ProcessListening(String),
    ProcesStopped
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct Settings {

}

#[derive(Debug, Default, Clone, Serialize)]
pub struct LoadResult {
    pub version: String,
    pub settings: Settings
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct Encounter {
    pub id: Uuid,
    pub updated_on: DateTime<Utc>,
    pub total_damage: i64
}