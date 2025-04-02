
use std::sync::Arc;
use log::info;
use lost_metrics_simulator::{GetSimulationTemplateCriteria, SimulationTemplate};
use tauri::{command, App, AppHandle, State};
use std::error::Error as StdError;
use crate::{app_ready_state::AppReadyState, error::AppError, models::{LoadResult, Settings}, settings_manager::{self, SettingsManager}};

#[command]
pub async fn get_simulation_templates(criteria: GetSimulationTemplateCriteria) -> Result<Vec<SimulationTemplate>, AppError> {

    let templates = vec![
        SimulationTemplate {
            id: "1".into(),
            name: "Brelshaza G1".into(),
            hp: 1e9 as u64,
            party_count: 2
        },
        SimulationTemplate {
            id: "2".into(),
            name: "Brelshaza G2".into(),
            hp: 1e9 as u64,
            party_count: 2
        }
    ];

    Ok(templates)
}
