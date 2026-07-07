use serde::Deserialize;
use skills_core::{
    Catalog, InstallCommandPreview, InstallationOutcome, Skill, SkillGroup, SkillsStore,
};
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

fn app_store(app: &AppHandle) -> Result<SkillsStore, String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    SkillsStore::open(data_dir).map_err(|error| error.to_string())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct CommandSkillInput {
    name: String,
    description: String,
    command: String,
    tags: Vec<String>,
}

#[tauri::command]
fn get_catalog(app: AppHandle) -> Result<Catalog, String> {
    let store = app_store(&app)?;
    Ok(store.catalog().clone())
}

#[tauri::command]
fn search_skills(app: AppHandle, query: String) -> Result<Vec<Skill>, String> {
    let store = app_store(&app)?;
    Ok(store.search(&query))
}

#[tauri::command]
fn import_path(app: AppHandle, path: String) -> Result<Vec<Skill>, String> {
    let mut store = app_store(&app)?;
    store.import_path(path).map_err(|error| error.to_string())
}

#[tauri::command]
fn add_command_skill(app: AppHandle, input: CommandSkillInput) -> Result<Skill, String> {
    let mut store = app_store(&app)?;
    store
        .add_command_skill(input.name, input.description, input.command, input.tags)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn delete_skill(app: AppHandle, skill_ref: String) -> Result<Skill, String> {
    let mut store = app_store(&app)?;
    store
        .delete_skill(&skill_ref)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn export_skill(
    app: AppHandle,
    skill_ref: String,
    output_dir: String,
    overwrite: bool,
) -> Result<String, String> {
    let store = app_store(&app)?;
    store
        .export_skill(&skill_ref, output_dir, overwrite)
        .map(|path| path.display().to_string())
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn create_group(
    app: AppHandle,
    name: String,
    description: Option<String>,
) -> Result<SkillGroup, String> {
    let mut store = app_store(&app)?;
    store
        .create_group(name, description)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn group_add_skill(
    app: AppHandle,
    group_ref: String,
    skill_ref: String,
) -> Result<SkillGroup, String> {
    let mut store = app_store(&app)?;
    store
        .add_skill_to_group(&group_ref, &skill_ref)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn group_remove_skill(
    app: AppHandle,
    group_ref: String,
    skill_ref: String,
) -> Result<SkillGroup, String> {
    let mut store = app_store(&app)?;
    store
        .remove_skill_from_group(&group_ref, &skill_ref)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn delete_group(app: AppHandle, group_ref: String) -> Result<SkillGroup, String> {
    let mut store = app_store(&app)?;
    store
        .delete_group(&group_ref)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn preview_skill_commands(
    app: AppHandle,
    skill_ref: String,
) -> Result<Vec<InstallCommandPreview>, String> {
    let store = app_store(&app)?;
    store
        .preview_skill_commands(&skill_ref)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn preview_group_commands(
    app: AppHandle,
    group_ref: String,
) -> Result<Vec<InstallCommandPreview>, String> {
    let store = app_store(&app)?;
    store
        .preview_group_commands(&group_ref)
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn install_skill(
    app: AppHandle,
    skill_ref: String,
    project_path: Option<String>,
    target_path: Option<String>,
    overwrite: bool,
    execute_commands: bool,
) -> Result<InstallationOutcome, String> {
    let store = app_store(&app)?;
    store
        .install_skill(
            &skill_ref,
            project_path.map(PathBuf::from),
            target_path.map(PathBuf::from),
            overwrite,
            execute_commands,
        )
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn install_group(
    app: AppHandle,
    group_ref: String,
    project_path: Option<String>,
    target_path: Option<String>,
    overwrite: bool,
    execute_commands: bool,
) -> Result<InstallationOutcome, String> {
    let store = app_store(&app)?;
    store
        .install_group(
            &group_ref,
            project_path.map(PathBuf::from),
            target_path.map(PathBuf::from),
            overwrite,
            execute_commands,
        )
        .map_err(|error| error.to_string())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            get_catalog,
            search_skills,
            import_path,
            add_command_skill,
            delete_skill,
            export_skill,
            create_group,
            group_add_skill,
            group_remove_skill,
            delete_group,
            preview_skill_commands,
            preview_group_commands,
            install_skill,
            install_group
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
