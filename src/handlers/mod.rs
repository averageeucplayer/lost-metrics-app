
use tauri::generate_handler;

mod load;
mod settings;
mod run_simulation;
mod get_simulation_templates;
mod get_stats;
mod get_past_encounters;

pub fn generate_handlers() -> Box<dyn Fn(tauri::ipc::Invoke) -> bool + Send + Sync> {
    Box::new(generate_handler![
        load::load,
        settings::get_settings,
        settings::save_settings,
        run_simulation::run_simulation,
        get_simulation_templates::get_simulation_templates,
        get_stats::get_stats,
        get_past_encounters::get_past_encounters
    ])
}
