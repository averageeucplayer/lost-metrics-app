use log::error;

pub fn set_hook() {
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
}