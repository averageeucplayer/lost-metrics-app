use std::time::Duration;

use chrono::{Date, DateTime, Utc};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum Message {
    Unknown,
    ProcessNotRunning,
    ProcessRunning,
    ProcessNotListening,
    ProcessListening(String),
    ProcesStopped
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnifferSettings {
    pub process_name: String,
    pub port: u16,
    #[serde(with = "humantime_serde")]
    pub timeout: Duration
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub version: VersionReq,
    pub sniffer: SnifferSettings
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct LoadResult {
    pub app_name: String,
    pub version: String,
    pub settings: Settings
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct Encounter {
    pub id: Uuid,
    pub updated_on: DateTime<Utc>,
    pub total_damage: i64
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RunSimulation {
    
}