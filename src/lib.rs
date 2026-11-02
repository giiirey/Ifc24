use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing::{error, info};

pub mod models;
pub mod services;
pub mod state;
mod commands;

#[tauri::command]
async fn open_file(app: tauri::AppHandle) -> Option<String> {
    use tauri_plugin_dialog::{DialogExt, FilePath};
    let path = app.dialog().file().blocking_pick_file();
    if let Some(p) = path {
        match p {
            FilePath::Path(pathbuf) => {
                let path_str = pathbuf.to_string_lossy().to_string();
                let app_state = crate::state::get_state();
                let mut guard = app_state.model.lock().ok()?;
                guard.model_path = Some(path_str.clone());
                return Some(path_str);
            }
            FilePath::Url(url) => {
                return Some(url.to_string());
            }
        }
    }
    None
}

#[tauri::command]
async fn open_ifc_file_from_dialog(app: tauri::AppHandle) -> Result<crate::models::ModelInfo, String> {
    eprintln!("[RUST] open_ifc_file_from_dialog called");
    use tauri_plugin_dialog::{DialogExt, FilePath};
    let state = crate::state::get_state();

    // Open file dialog and get path
    eprintln!("[RUST] Opening file picker...");
    let selected = app.dialog().file().blocking_pick_file()
        .ok_or("No file selected")?;
    
    let file_path_str = match selected {
        FilePath::Path(p) => p.to_string_lossy().to_string(),
        FilePath::Url(u) => u.to_string(),
    };

    // Save to global state so other commands can access it
    {
        state.model.lock()
            .map_err(|e| e.to_string())?
            .model_path = Some(file_path_str.clone());
    }

    crate::commands::file_commands::open_ifc_file(file_path_str).await
}

#[tauri::command]
async fn close_model() -> Result<(), String> {
    let app = state::get_state();
    let mut state = app.model.lock().map_err(|e| e.to_string())?;
    state.reset();
    Ok(())
}

pub fn run() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_target(false))
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ifctauri=debug,tower_web=info".into()),
        )
        .init();

    info!("IFC Tauri starting");

    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            open_file,
            open_ifc_file_from_dialog,
            close_model,
            commands::file_commands::open_ifc_file,
            commands::model_commands::get_entity_count,
            commands::model_commands::get_entity_by_id,
            commands::model_commands::get_entity_property_sets,
            commands::model_commands::get_hierarchy_root,
            commands::model_commands::get_children,
            commands::model_commands::search_entities,
            commands::model_commands::get_entity_types,
            commands::model_commands::get_all_entity_ids,
            commands::model_commands::get_all_entities,
            commands::geometry_commands::load_geometry_batch,
            commands::geometry_commands::release_geometry,
            commands::geometry_commands::get_bounding_box,
            commands::geometry_commands::get_material_properties,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
