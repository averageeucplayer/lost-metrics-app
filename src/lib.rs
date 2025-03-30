#![allow(warnings)]

use handlers::generate_handlers;
use tauri::generate_context;
use tauri_plugin_log::{Target, TargetKind};
use tauri::Context;
mod handlers;
mod setup;
mod hook;
mod updater;
mod process_watcher;
mod models;
mod aws_iprange;
mod processor;
mod app_ready_state;
mod settings_manager;
mod error;

pub fn run() {
    hook::set_hook();

    let context: Context = generate_context!();

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
