
use std::sync::Arc;
use log::info;
use tauri::{command, App, AppHandle, State};
use tokio::sync::Mutex;
use std::error::Error as StdError;
use crate::{app_ready_state::AppReadyState, error::AppError, models::{LoadResult, Settings}, settings_manager::{self, SettingsManager}};

#[command]
pub async fn get_settings(
    settings_manager: State<'_, Arc<Mutex<SettingsManager>>>) -> Result<Settings, AppError> {

    let mut settings_manager = settings_manager.lock().await;
    let settings = settings_manager.get_or_create_default().await?;
    
    Ok(settings)
}


#[command]
pub async fn save_settings(
    settings_manager: State<'_, Arc<Mutex<SettingsManager>>>, settings: Settings) -> Result<(), AppError> {

    let mut settings_manager = settings_manager.lock().await;
    settings_manager.save(&settings).await?;
    
    Ok(())
}
