use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use tracing::{error, info};

pub mod models;
pub mod services;
pub mod state;
mod commands;

#[tauri::command]
async fn open_file(app: tauri::AppHandle) -> Option<String> {
    use tauri_plugin_dialog::DialogExt;
    let path = app.dialog().file().blocking_pick_file();
    path.map(|p| p.to_string().to_string())
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
            close_model,
            commands::file_commands::open_ifc_file,
            commands::model_commands::get_entity_count,
            commands::model_commands::get_entity_by_id,
            commands::model_commands::get_hierarchy_root,
            commands::model_commands::get_children,
            commands::model_commands::search_entities,
            commands::model_commands::get_entity_types,
            commands::geometry_commands::load_geometry_batch,
            commands::geometry_commands::release_geometry,
            commands::geometry_commands::get_bounding_box,
            commands::geometry_commands::get_material_properties,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
