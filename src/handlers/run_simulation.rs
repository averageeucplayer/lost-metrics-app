
use std::sync::Arc;
use log::info;
use lost_metrics_simulator::{simulator::Simulator, NewSimulation};
use tauri::{command, App, AppHandle, State};
use std::error::Error as StdError;
use crate::{app_ready_state::AppReadyState, error::AppError, models::{LoadResult, Settings}, settings_manager::{self, SettingsManager}};

#[command]
pub async fn run_simulation(
    simulator: State<'_, Arc<Simulator>>, 
    payload: NewSimulation) -> Result<(), AppError> {

    Ok(())
}
