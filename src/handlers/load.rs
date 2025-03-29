
use std::sync::Arc;

use tauri::{command, AppHandle, State};

use crate::{app_ready_state::AppReadyState, models::{LoadResult, Settings}};

#[command]
pub fn load(state: State<Arc<AppReadyState>>, app_handle: AppHandle) -> LoadResult {
    state.mark_ready();

    let version = app_handle.package_info().version.to_string();

    LoadResult {
        version,
        settings: Settings {

        }
    }
}
