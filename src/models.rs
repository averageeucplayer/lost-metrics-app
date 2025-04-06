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
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
pub struct Encounter {
    pub id: Uuid,
    pub updated_on: DateTime<Utc>,
    pub participants: Vec<Player>,
    pub boss: Boss,
    pub total_damage: FormattedValue,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct FormattedValue {
    pub raw: u64,
    pub value: f64,
    pub unit: &'static str,
    pub formatted: String,
}

impl From<u64> for FormattedValue {
    fn from(n: u64) -> Self {
        const UNITS: &[(&str, u64)] = &[
            ("T", 1_000_000_000_000),
            ("B", 1_000_000_000),
            ("M", 1_000_000),
            ("k", 1_000),
        ];

        for (unit, threshold) in UNITS {
            if n >= *threshold {
                let value = (n as f64) / (*threshold as f64);
                let formatted = format!("{:.1}{}", value, unit);
                return Self {
                    raw: n,
                    value: (n as f64) / (*threshold as f64),
                    unit,
                    formatted
                };
            }
        }

        let formatted = n.to_string();
        Self {
            raw: n,
            value: n as f64,
            unit: "",
            formatted,
        }
    }
}

use std::ops::AddAssign;

impl AddAssign<u64> for FormattedValue {
    fn add_assign(&mut self, rhs: u64) {
        self.raw += rhs;

        const UNITS: &[(&str, u64)] = &[
            ("T", 1_000_000_000_000),
            ("B", 1_000_000_000),
            ("M", 1_000_000),
            ("k", 1_000),
        ];

        for (unit, threshold) in UNITS {
            if self.raw >= *threshold {
                self.value = (self.raw as f64) / (*threshold as f64);
                self.unit = unit;
                self.formatted = format!("{:.1}{}", self.value, unit);
                return;
            }
        }

        self.value = self.raw as f64;
        self.unit = "";
        self.formatted = self.raw.to_string();
    }
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct Player {
    pub id: u64,
    pub name: String,
    pub class_id: u32,
    pub class_name: String,
    pub stats: PlayerStats
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct PlayerStats {
    pub total_damage: FormattedValue,
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct Boss {
    pub id: u64,
    pub name: String,
}


#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RunSimulation {
    
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GetStatsResult {
    pub class_popularity: Vec<Metric>,
    pub item_level_breakdown: Vec<Metric>,
    pub server_population: ServerPopulation,
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
