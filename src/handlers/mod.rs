
use tauri::generate_handler;

mod load;
mod save_settings;
mod run_simulation;
mod get_simulation_templates;

pub fn generate_handlers() -> Box<dyn Fn(tauri::ipc::Invoke) -> bool + Send + Sync> {
    Box::new(generate_handler![
        load::load,
        save_settings::get_settings,
        save_settings::save_settings,
        run_simulation::run_simulation,
        get_simulation_templates::get_simulation_templates
    ])
}
