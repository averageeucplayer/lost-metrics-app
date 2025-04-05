
use std::sync::Arc;
use log::info;
use tauri::{command, App, AppHandle, State};
use std::error::Error as StdError;
use crate::{app_ready_state::AppReadyState, error::AppError, models::{Encounter, GetPastEncountersCriteria, LoadResult, Settings}, settings_manager::{self, SettingsManager}};

#[command]
pub async fn get_past_encounters(criteria: GetPastEncountersCriteria) -> Result<Vec<Encounter>, AppError> {

    let encounters = vec![
       
    ];

    Ok(encounters)
}
