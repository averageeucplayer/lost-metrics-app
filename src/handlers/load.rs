
use std::sync::Arc;
use tauri::{command, App, AppHandle, State};
use std::error::Error as StdError;
use crate::{app_ready_state::AppReadyState, error::AppError, models::{LoadResult, Settings}, settings_manager::{self, SettingsManager}};

#[command]
pub async fn load(
    state: State<'_, Arc<AppReadyState>>,
    app_handle: AppHandle) -> Result<LoadResult, AppError> {
    state.mark_ready();

    let version = app_handle.package_info().version.to_string();

    let result = LoadResult {
        app_name: "Lost Metrics".into(),
        version,
        github_link: "".into()
        // settings,
    };

    Ok(result)
}
