use serde::Deserialize;
use skills_core::{
    Catalog, CommandExecutionMode, CommandInputStep, CommandMode, InstallCommandPreview,
    InstallationOutcome, Skill, SkillGroup, SkillsStore,
};
use std::path::PathBuf;
use std::process::Command;
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
    #[serde(default)]
    command_mode: CommandMode,
    #[serde(default)]
    command_input_steps: Vec<CommandInputStep>,
    tags: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct UpdateCommandSkillInput {
    skill_ref: String,
    command_mode: Option<CommandMode>,
    command_input_steps: Option<Vec<CommandInputStep>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstallSkillInput {
    skill_ref: String,
    project_path: Option<String>,
    target_path: Option<String>,
    overwrite: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct InstallGroupInput {
    group_ref: String,
    project_path: Option<String>,
    target_path: Option<String>,
    overwrite: bool,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OpenCommandTerminalInput {
    command: String,
    project_path: Option<String>,
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
        .add_command_skill(
            input.name,
            input.description,
            input.command,
            input.command_mode,
            input.command_input_steps,
            input.tags,
        )
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn update_command_skill(app: AppHandle, input: UpdateCommandSkillInput) -> Result<Skill, String> {
    let mut store = app_store(&app)?;
    store
        .update_command_skill(
            &input.skill_ref,
            input.command_mode,
            input.command_input_steps,
        )
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
fn install_skill(app: AppHandle, input: InstallSkillInput) -> Result<InstallationOutcome, String> {
    let store = app_store(&app)?;
    store
        .install_skill(
            &input.skill_ref,
            input.project_path.map(PathBuf::from),
            input.target_path.map(PathBuf::from),
            input.overwrite,
            CommandExecutionMode::Captured,
        )
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn install_group(app: AppHandle, input: InstallGroupInput) -> Result<InstallationOutcome, String> {
    let store = app_store(&app)?;
    store
        .install_group(
            &input.group_ref,
            input.project_path.map(PathBuf::from),
            input.target_path.map(PathBuf::from),
            input.overwrite,
            CommandExecutionMode::Captured,
        )
        .map_err(|error| error.to_string())
}

#[tauri::command]
fn open_command_terminal(input: OpenCommandTerminalInput) -> Result<(), String> {
    let mut process = if cfg!(windows) {
        let mut command = Command::new("powershell.exe");
        command.args(["-NoExit", "-Command", &input.command]);
        command
    } else {
        let mut command = Command::new("sh");
        command.args(["-c", &input.command]);
        command
    };

    if let Some(project_path) = input.project_path.filter(|path| !path.trim().is_empty()) {
        process.current_dir(project_path);
    }

    process
        .spawn()
        .map(|_| ())
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
            update_command_skill,
            delete_skill,
            export_skill,
            create_group,
            group_add_skill,
            group_remove_skill,
            delete_group,
            preview_skill_commands,
            preview_group_commands,
            install_skill,
            install_group,
            open_command_terminal
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
