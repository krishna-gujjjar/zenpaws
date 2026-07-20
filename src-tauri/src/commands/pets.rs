use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};
use uuid::Uuid;
use zenpaws_pets::PetRegistry;

/// Details returned after a transparent pet overlay is created.
#[derive(serde::Serialize)]
pub struct PetWindowStatus {
    pub label: String,
}

/// Opens one transparent, click-through pet overlay for an installed package.
#[tauri::command]
pub fn open_pet_window(app: AppHandle, manifest_id: String) -> Result<PetWindowStatus, String> {
    let assets = app
        .path()
        .resource_dir()
        .map_err(|error| error.to_string())?
        .join("assets")
        .join("pets");
    let registry = PetRegistry::load(assets).map_err(|error| error.to_string())?;
    if registry.get(&manifest_id).is_none() {
        return Err("requested pet package is not installed".to_owned());
    }

    let label = format!("pet-{manifest_id}-{}", Uuid::new_v4());
    let window = WebviewWindowBuilder::new(&app, &label, WebviewUrl::App("index.html".into()))
        .always_on_top(true)
        .decorations(false)
        .resizable(false)
        .skip_taskbar(true)
        .transparent(true)
        .build()
        .map_err(|error| error.to_string())?;
    window
        .set_ignore_cursor_events(true)
        .map_err(|error| error.to_string())?;
    Ok(PetWindowStatus { label })
}

/// Closes only a pet-scoped overlay window.
#[tauri::command]
pub fn close_pet_window(app: AppHandle, label: String) -> Result<(), String> {
    if !label.starts_with("pet-") {
        return Err("window label is outside the pet scope".to_owned());
    }
    let window = app
        .get_webview_window(&label)
        .ok_or_else(|| "pet window does not exist".to_owned())?;
    window.close().map_err(|error| error.to_string())
}
