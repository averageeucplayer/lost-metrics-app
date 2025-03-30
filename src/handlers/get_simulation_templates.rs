
use std::sync::Arc;
use log::info;
use tauri::{command, App, AppHandle, State};
use std::error::Error as StdError;
use crate::{app_ready_state::AppReadyState, error::AppError, models::{LoadResult, Settings}, settings_manager::{self, SettingsManager}};

#[command]
pub async fn get_simulation_templates() -> Result<(), AppError> {

    Ok(())
}
