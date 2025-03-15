
use tauri::generate_handler;

mod load;


pub fn generate_handlers() -> Box<dyn Fn(tauri::ipc::Invoke) -> bool + Send + Sync> {
    Box::new(generate_handler![
        load::load
    ])
}
