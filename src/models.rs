use std::time::Duration;

use chrono::{Date, DateTime, Utc};
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "type", content = "message")]
pub enum ProcessState {
    Unknown,
    ProcessNotRunning,
    ProcessRunning,
    ProcessNotListening,
    ProcessListening(String),
    ProcesStopped
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "type", content = "message")]
pub enum UpdaterState {
    Unknown,
    NewVersion,
    LatestVersion,
    Error(String)
}

#[derive(Debug, Clone, Serialize)]
pub struct ProcessWatcherResult {
    pub checked_on: DateTime<Utc>,
    pub state: ProcessState
}

#[derive(Debug, Clone, Serialize)]
pub struct UpdaterResult {
    pub checked_on: DateTime<Utc>,
    pub state: UpdaterState
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SnifferSettings {
    pub process_name: String,
    pub port: u16,
    #[serde(with = "humantime_serde")]
    pub check_interval: Duration
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub version: VersionReq,
    pub sniffer: SnifferSettings
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoadResult {
    pub app_name: String,
    pub github_link: String,
    pub version: String,
    // pub settings: Settings
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct Encounter {
    pub id: Uuid,
    pub updated_on: DateTime<Utc>,
    pub total_damage: i64,
    pub participants: Vec<Player>,
    pub boss: Boss
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct Player {
    pub id: u64,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct Boss {
    pub id: u64,
}


#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RunSimulation {
    
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetStatsResult {
    pub class_popularity: Vec<Metric>,
    pub item_level_breakdown: Vec<Metric>,
    pub server_population: Vec<Metric>,
    pub metrics: Vec<Metric>
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerPopulation {
    pub na: NorthAmericaNode,
    pub eu: EuropeNode
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct NorthAmericaNode {
    pub name: String,
    pub naw: Vec<Metric>,
    pub nae: Vec<Metric>
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct EuropeNode {
    pub name: String,
    pub metrics: Vec<Metric>
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Metric {
    pub name: String,
    pub value: f32
}


#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetPastEncountersCriteria {
    
}
