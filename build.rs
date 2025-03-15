fn main() {

    let mut window_attributes = tauri_build::WindowsAttributes::new();

    if !cfg!(debug_assertions) {
        let manifest = include_str!("./require_admin.manifest");
        window_attributes = window_attributes.app_manifest(manifest);
    }

    let attributes = tauri_build::Attributes::new().windows_attributes(window_attributes);
    
    tauri_build::try_build(attributes).expect("Failed to run build script");
}
