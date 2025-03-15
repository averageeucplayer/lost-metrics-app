use handlers::generate_handlers;
use log::error;
use tauri::generate_context;
use tauri_plugin_log::{Target, TargetKind};

mod handlers;
mod setup;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    std::panic::set_hook(Box::new(|info| {
        let payload = info.payload();
        let message = if let Some(s) = payload.downcast_ref::<&str>() {
            s.to_string()
        } else if let Some(s) = payload.downcast_ref::<String>() {
            s.clone()
        } else {
            "Unknown panic message".to_string()
        };

        let location = info.location().map_or("unknown location".to_string(), |location| {
            format!("{}:{}", location.file(), location.line())
        });

        error!("Panicked at '{}', {}", message, location);
    }));

    let context = generate_context!();

    tauri::Builder::default()
            .plugin(tauri_plugin_log::Builder::new()
            .level(log::LevelFilter::Debug)
            .targets([
                Target::new(TargetKind::Stdout),
                Target::new(TargetKind::LogDir {
                    file_name: Some("logs".to_string()),
                })
            ])
            .build())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_single_instance::init(|_, _, _| {}))
        .plugin(tauri_plugin_updater::Builder::new().build())
        .invoke_handler(generate_handlers())
        .setup(setup::setup_app)
        .run(context)
        .expect("error while running tauri application");
}
