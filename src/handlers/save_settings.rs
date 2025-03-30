
use std::sync::Arc;
use log::info;
use tauri::{command, App, AppHandle, State};
use std::error::Error as StdError;
use crate::{app_ready_state::AppReadyState, error::AppError, models::{LoadResult, Settings}, settings_manager::{self, SettingsManager}};

#[command]
pub async fn save_settings(
    settings_manager: State<'_, Arc<SettingsManager>>, settings: Settings) -> Result<(), AppError> {

    settings_manager.save(&settings).await?;
    
    Ok(())
}
