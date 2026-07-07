use directories::BaseDirs;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

pub const SCHEMA_VERSION: u32 = 1;
pub const SKILL_FILE_NAME: &str = "SKILL.md";
pub const APP_IDENTIFIER: &str = "dev.skillslist.app";

#[derive(Debug, Error)]
pub enum SkillsError {
    #[error("io error: {0}")]
    Io(#[from] io::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("yaml error: {0}")]
    Yaml(#[from] serde_yaml::Error),
    #[error("skill not found: {0}")]
    SkillNotFound(String),
    #[error("group not found: {0}")]
    GroupNotFound(String),
    #[error("invalid skill folder: {0}")]
    InvalidSkillFolder(String),
    #[error("target already exists: {0}")]
    TargetExists(String),
    #[error("install command failed: {0}")]
    CommandFailed(String),
    #[error("unable to find a data directory for this platform")]
    MissingDataDir,
}

pub type Result<T> = std::result::Result<T, SkillsError>;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SourceType {
    Local,
    Command,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Skill {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: Option<String>,
    pub source_type: SourceType,
    pub library_path: Option<PathBuf>,
    pub install_command: Option<String>,
    pub tags: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct SkillGroup {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub skill_ids: Vec<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct Catalog {
    pub schema_version: u32,
    pub skills: Vec<Skill>,
    pub groups: Vec<SkillGroup>,
}

impl Default for Catalog {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            skills: Vec::new(),
            groups: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InstallCommandPreview {
    pub skill_id: String,
    pub skill_name: String,
    pub command: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InstalledSkill {
    pub skill_id: String,
    pub skill_name: String,
    pub target_path: Option<PathBuf>,
    pub command: Option<String>,
    pub executed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct InstallationOutcome {
    pub target_dir: PathBuf,
    pub installed_skills: Vec<InstalledSkill>,
    pub command_previews: Vec<InstallCommandPreview>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CommandExecutionMode {
    PreviewOnly,
    Captured,
    InteractiveTerminal,
}

#[derive(Clone, Debug)]
pub struct AppPaths {
    pub data_dir: PathBuf,
    pub catalog_path: PathBuf,
    pub library_dir: PathBuf,
    pub skills_dir: PathBuf,
}

impl AppPaths {
    pub fn from_data_dir(data_dir: impl Into<PathBuf>) -> Self {
        let data_dir = data_dir.into();
        let catalog_path = data_dir.join("catalog.json");
        let library_dir = data_dir.join("library");
        let skills_dir = library_dir.join("skills");

        Self {
            data_dir,
            catalog_path,
            library_dir,
            skills_dir,
        }
    }

    pub fn default_data_dir() -> Result<PathBuf> {
        let dirs = BaseDirs::new().ok_or(SkillsError::MissingDataDir)?;
        Ok(dirs.data_dir().join(APP_IDENTIFIER))
    }
}

pub struct SkillsStore {
    pub paths: AppPaths,
    catalog: Catalog,
}

impl SkillsStore {
    pub fn open_default() -> Result<Self> {
        Self::open(AppPaths::default_data_dir()?)
    }

    pub fn open(data_dir: impl Into<PathBuf>) -> Result<Self> {
        let paths = AppPaths::from_data_dir(data_dir);
        fs::create_dir_all(&paths.skills_dir)?;

        let catalog = if paths.catalog_path.exists() {
            let raw = fs::read_to_string(&paths.catalog_path)?;
            serde_json::from_str(&raw)?
        } else {
            Catalog::default()
        };

        Ok(Self { paths, catalog })
    }

    pub fn catalog(&self) -> &Catalog {
        &self.catalog
    }

    pub fn save(&self) -> Result<()> {
        fs::create_dir_all(&self.paths.data_dir)?;
        let raw = serde_json::to_string_pretty(&self.catalog)?;
        fs::write(&self.paths.catalog_path, raw)?;
        Ok(())
    }

    pub fn search(&self, query: &str) -> Vec<Skill> {
        let needle = query.trim().to_lowercase();
        if needle.is_empty() {
            return self.catalog.skills.clone();
        }

        self.catalog
            .skills
            .iter()
            .filter(|skill| {
                skill.name.to_lowercase().contains(&needle)
                    || skill.description.to_lowercase().contains(&needle)
                    || skill.id.to_lowercase().contains(&needle)
                    || skill
                        .tags
                        .iter()
                        .any(|tag| tag.to_lowercase().contains(&needle))
            })
            .cloned()
            .collect()
    }

    pub fn import_path(&mut self, source: impl AsRef<Path>) -> Result<Vec<Skill>> {
        let source = source.as_ref();
        let folders = discover_skill_folders(source)?;
        if folders.is_empty() {
            return Err(SkillsError::InvalidSkillFolder(
                source.display().to_string(),
            ));
        }

        let mut imported = Vec::new();
        for folder in folders {
            let mut parsed = parse_skill_folder(&folder)?;
            parsed.id = self.unique_skill_id(&parsed.id);
            let destination = self.paths.skills_dir.join(&parsed.id);
            copy_dir_all(&folder, &destination)?;
            parsed.library_path = Some(destination);
            parsed.source_type = SourceType::Local;
            parsed.install_command = None;
            self.catalog.skills.push(parsed.clone());
            imported.push(parsed);
        }

        self.save()?;
        Ok(imported)
    }

    pub fn add_command_skill(
        &mut self,
        name: String,
        description: String,
        command: String,
        tags: Vec<String>,
    ) -> Result<Skill> {
        let id = self.unique_skill_id(&slugify(&name));
        let skill = Skill {
            id,
            name,
            description,
            version: None,
            source_type: SourceType::Command,
            library_path: None,
            install_command: Some(command),
            tags,
        };

        self.catalog.skills.push(skill.clone());
        self.save()?;
        Ok(skill)
    }

    pub fn delete_skill(&mut self, skill_ref: &str) -> Result<Skill> {
        let index = self
            .catalog
            .skills
            .iter()
            .position(|skill| matches_skill(skill, skill_ref))
            .ok_or_else(|| SkillsError::SkillNotFound(skill_ref.to_string()))?;
        let skill = self.catalog.skills.remove(index);

        for group in &mut self.catalog.groups {
            group.skill_ids.retain(|id| id != &skill.id);
        }

        if let Some(path) = &skill.library_path {
            if path.exists() && path.starts_with(&self.paths.skills_dir) {
                fs::remove_dir_all(path)?;
            }
        }

        self.save()?;
        Ok(skill)
    }

    pub fn export_skill(
        &self,
        skill_ref: &str,
        output_dir: impl AsRef<Path>,
        overwrite: bool,
    ) -> Result<PathBuf> {
        let skill = self.find_skill(skill_ref)?;
        let Some(source) = &skill.library_path else {
            return Err(SkillsError::InvalidSkillFolder(format!(
                "{} has no local folder to export",
                skill.name
            )));
        };

        let destination = output_dir.as_ref().join(&skill.id);
        if destination.exists() {
            if !overwrite {
                return Err(SkillsError::TargetExists(destination.display().to_string()));
            }
            fs::remove_dir_all(&destination)?;
        }
        copy_dir_all(source, &destination)?;
        Ok(destination)
    }

    pub fn create_group(
        &mut self,
        name: String,
        description: Option<String>,
    ) -> Result<SkillGroup> {
        let group = SkillGroup {
            id: self.unique_group_id(&slugify(&name)),
            name,
            description,
            skill_ids: Vec::new(),
        };
        self.catalog.groups.push(group.clone());
        self.save()?;
        Ok(group)
    }

    pub fn add_skill_to_group(&mut self, group_ref: &str, skill_ref: &str) -> Result<SkillGroup> {
        let skill_id = self.find_skill(skill_ref)?.id.clone();
        let group = self.find_group_mut(group_ref)?;
        if !group.skill_ids.contains(&skill_id) {
            group.skill_ids.push(skill_id);
        }
        let group = group.clone();
        self.save()?;
        Ok(group)
    }

    pub fn remove_skill_from_group(
        &mut self,
        group_ref: &str,
        skill_ref: &str,
    ) -> Result<SkillGroup> {
        let skill_id = self.find_skill(skill_ref)?.id.clone();
        let group = self.find_group_mut(group_ref)?;
        group.skill_ids.retain(|id| id != &skill_id);
        let group = group.clone();
        self.save()?;
        Ok(group)
    }

    pub fn delete_group(&mut self, group_ref: &str) -> Result<SkillGroup> {
        let index = self
            .catalog
            .groups
            .iter()
            .position(|group| matches_group(group, group_ref))
            .ok_or_else(|| SkillsError::GroupNotFound(group_ref.to_string()))?;
        let group = self.catalog.groups.remove(index);
        self.save()?;
        Ok(group)
    }

    pub fn install_skill(
        &self,
        skill_ref: &str,
        project_path: Option<PathBuf>,
        target_path: Option<PathBuf>,
        overwrite: bool,
        command_mode: CommandExecutionMode,
    ) -> Result<InstallationOutcome> {
        let skill = self.find_skill(skill_ref)?.clone();
        let target_dir = resolve_target_dir(project_path.as_deref(), target_path.as_deref())?;
        let mut outcome = InstallationOutcome {
            target_dir,
            installed_skills: Vec::new(),
            command_previews: Vec::new(),
        };
        self.install_one(
            &skill,
            project_path.as_deref(),
            overwrite,
            command_mode,
            &mut outcome,
        )?;
        Ok(outcome)
    }

    pub fn preview_skill_commands(&self, skill_ref: &str) -> Result<Vec<InstallCommandPreview>> {
        let skill = self.find_skill(skill_ref)?;
        Ok(command_preview_for_skill(skill).into_iter().collect())
    }

    pub fn preview_group_commands(&self, group_ref: &str) -> Result<Vec<InstallCommandPreview>> {
        let group = self.find_group(group_ref)?;
        let mut previews = Vec::new();
        for skill_id in &group.skill_ids {
            let skill = self.find_skill(skill_id)?;
            if let Some(preview) = command_preview_for_skill(skill) {
                previews.push(preview);
            }
        }
        Ok(previews)
    }

    pub fn install_group(
        &self,
        group_ref: &str,
        project_path: Option<PathBuf>,
        target_path: Option<PathBuf>,
        overwrite: bool,
        command_mode: CommandExecutionMode,
    ) -> Result<InstallationOutcome> {
        let group = self.find_group(group_ref)?;
        let target_dir = resolve_target_dir(project_path.as_deref(), target_path.as_deref())?;
        let mut outcome = InstallationOutcome {
            target_dir,
            installed_skills: Vec::new(),
            command_previews: Vec::new(),
        };

        for skill_id in &group.skill_ids {
            let skill = self.find_skill(skill_id)?.clone();
            self.install_one(
                &skill,
                project_path.as_deref(),
                overwrite,
                command_mode,
                &mut outcome,
            )?;
        }

        Ok(outcome)
    }

    fn install_one(
        &self,
        skill: &Skill,
        project_path: Option<&Path>,
        overwrite: bool,
        command_mode: CommandExecutionMode,
        outcome: &mut InstallationOutcome,
    ) -> Result<()> {
        match skill.source_type {
            SourceType::Local => {
                let Some(source) = &skill.library_path else {
                    return Err(SkillsError::InvalidSkillFolder(format!(
                        "{} has no local folder",
                        skill.name
                    )));
                };
                fs::create_dir_all(&outcome.target_dir)?;
                let destination = outcome.target_dir.join(&skill.id);
                if destination.exists() {
                    if !overwrite {
                        return Err(SkillsError::TargetExists(destination.display().to_string()));
                    }
                    if destination.starts_with(&outcome.target_dir) {
                        fs::remove_dir_all(&destination)?;
                    }
                }
                copy_dir_all(source, &destination)?;
                outcome.installed_skills.push(InstalledSkill {
                    skill_id: skill.id.clone(),
                    skill_name: skill.name.clone(),
                    target_path: Some(destination),
                    command: None,
                    executed: true,
                });
            }
            SourceType::Command => {
                let command = skill.install_command.clone().unwrap_or_default();
                match command_mode {
                    CommandExecutionMode::PreviewOnly => {
                        outcome.command_previews.push(InstallCommandPreview {
                            skill_id: skill.id.clone(),
                            skill_name: skill.name.clone(),
                            command,
                        });
                    }
                    CommandExecutionMode::Captured => {
                        run_install_command_captured(&command, project_path)?;
                        outcome.installed_skills.push(InstalledSkill {
                            skill_id: skill.id.clone(),
                            skill_name: skill.name.clone(),
                            target_path: None,
                            command: Some(command),
                            executed: true,
                        });
                    }
                    CommandExecutionMode::InteractiveTerminal => {
                        run_install_command_interactive(&command, project_path)?;
                        outcome.installed_skills.push(InstalledSkill {
                            skill_id: skill.id.clone(),
                            skill_name: skill.name.clone(),
                            target_path: None,
                            command: Some(command),
                            executed: true,
                        });
                    }
                }
            }
        }
        Ok(())
    }

    fn find_skill(&self, skill_ref: &str) -> Result<&Skill> {
        self.catalog
            .skills
            .iter()
            .find(|skill| matches_skill(skill, skill_ref))
            .ok_or_else(|| SkillsError::SkillNotFound(skill_ref.to_string()))
    }

    fn find_group(&self, group_ref: &str) -> Result<&SkillGroup> {
        self.catalog
            .groups
            .iter()
            .find(|group| matches_group(group, group_ref))
            .ok_or_else(|| SkillsError::GroupNotFound(group_ref.to_string()))
    }

    fn find_group_mut(&mut self, group_ref: &str) -> Result<&mut SkillGroup> {
        self.catalog
            .groups
            .iter_mut()
            .find(|group| matches_group(group, group_ref))
            .ok_or_else(|| SkillsError::GroupNotFound(group_ref.to_string()))
    }

    fn unique_skill_id(&self, base: &str) -> String {
        unique_id(
            base,
            self.catalog.skills.iter().map(|skill| skill.id.as_str()),
        )
    }

    fn unique_group_id(&self, base: &str) -> String {
        unique_id(
            base,
            self.catalog.groups.iter().map(|group| group.id.as_str()),
        )
    }
}

#[derive(Debug, Deserialize)]
struct SkillFrontmatter {
    name: Option<String>,
    description: Option<String>,
    version: Option<String>,
    tags: Option<Vec<String>>,
}

pub fn parse_skill_folder(folder: impl AsRef<Path>) -> Result<Skill> {
    let folder = folder.as_ref();
    let skill_file = folder.join(SKILL_FILE_NAME);
    if !skill_file.is_file() {
        return Err(SkillsError::InvalidSkillFolder(
            folder.display().to_string(),
        ));
    }

    let raw = fs::read_to_string(&skill_file)?;
    let frontmatter = parse_frontmatter(&raw)?;
    let name = frontmatter
        .name
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| {
            folder
                .file_name()
                .map(|name| name.to_string_lossy().to_string())
                .unwrap_or_else(|| "skill".to_string())
        });

    Ok(Skill {
        id: slugify(&name),
        name,
        description: frontmatter.description.unwrap_or_default(),
        version: frontmatter.version,
        source_type: SourceType::Local,
        library_path: Some(folder.to_path_buf()),
        install_command: None,
        tags: frontmatter.tags.unwrap_or_default(),
    })
}

fn parse_frontmatter(raw: &str) -> Result<SkillFrontmatter> {
    let normalized = raw.replace("\r\n", "\n");
    if !normalized.starts_with("---\n") {
        return Ok(SkillFrontmatter {
            name: None,
            description: None,
            version: None,
            tags: None,
        });
    }

    let Some(end) = normalized[4..].find("\n---") else {
        return Ok(SkillFrontmatter {
            name: None,
            description: None,
            version: None,
            tags: None,
        });
    };
    let yaml = &normalized[4..4 + end];
    Ok(serde_yaml::from_str(yaml)?)
}

pub fn discover_skill_folders(root: impl AsRef<Path>) -> Result<Vec<PathBuf>> {
    let root = root.as_ref();
    if root.join(SKILL_FILE_NAME).is_file() {
        return Ok(vec![root.to_path_buf()]);
    }

    let mut folders = Vec::new();
    discover_skill_folders_rec(root, &mut folders)?;
    folders.sort();
    Ok(folders)
}

fn discover_skill_folders_rec(path: &Path, folders: &mut Vec<PathBuf>) -> Result<()> {
    if !path.is_dir() {
        return Ok(());
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let child = entry.path();
        if !child.is_dir() {
            continue;
        }
        if child.join(SKILL_FILE_NAME).is_file() {
            folders.push(child);
        } else {
            discover_skill_folders_rec(&child, folders)?;
        }
    }

    Ok(())
}

fn resolve_target_dir(project_path: Option<&Path>, target_path: Option<&Path>) -> Result<PathBuf> {
    if let Some(target_path) = target_path {
        return Ok(target_path.to_path_buf());
    }

    let project = match project_path {
        Some(path) => path.to_path_buf(),
        None => std::env::current_dir()?,
    };

    Ok(project.join(".agents").join("skills"))
}

fn install_command_process(command: &str, project_path: Option<&Path>) -> Command {
    let mut process = if cfg!(windows) {
        let mut command_process = Command::new("cmd");
        command_process.args(["/C", command]);
        command_process
    } else {
        let mut command_process = Command::new("sh");
        command_process.args(["-c", command]);
        command_process
    };

    if let Some(project_path) = project_path {
        process.current_dir(project_path);
    }

    process
}

fn run_install_command_interactive(command: &str, project_path: Option<&Path>) -> Result<()> {
    let mut process = install_command_process(command, project_path);
    let status = process.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(SkillsError::CommandFailed(format!(
            "command exited with status {status}"
        )))
    }
}

fn run_install_command_captured(command: &str, project_path: Option<&Path>) -> Result<()> {
    let mut process = install_command_process(command, project_path);
    let output = process.output()?;
    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(SkillsError::CommandFailed(stderr.trim().to_string()))
    }
}

fn copy_dir_all(source: &Path, destination: &Path) -> Result<()> {
    fs::create_dir_all(destination)?;
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        let from = entry.path();
        let to = destination.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir_all(&from, &to)?;
        } else if file_type.is_file() {
            fs::copy(&from, &to)?;
        }
    }
    Ok(())
}

fn slugify(input: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;

    for character in input.chars() {
        if character.is_ascii_alphanumeric() {
            slug.push(character.to_ascii_lowercase());
            last_dash = false;
        } else if !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }

    let trimmed = slug.trim_matches('-').to_string();
    if trimmed.is_empty() {
        "skill".to_string()
    } else {
        trimmed
    }
}

fn unique_id<'a>(base: &str, existing: impl Iterator<Item = &'a str>) -> String {
    let existing: HashSet<&str> = existing.collect();
    if !existing.contains(base) {
        return base.to_string();
    }

    for index in 2.. {
        let candidate = format!("{base}-{index}");
        if !existing.contains(candidate.as_str()) {
            return candidate;
        }
    }

    unreachable!("unique id loop is unbounded")
}

fn matches_skill(skill: &Skill, skill_ref: &str) -> bool {
    skill.id == skill_ref || skill.name.eq_ignore_ascii_case(skill_ref)
}

fn matches_group(group: &SkillGroup, group_ref: &str) -> bool {
    group.id == group_ref || group.name.eq_ignore_ascii_case(group_ref)
}

fn command_preview_for_skill(skill: &Skill) -> Option<InstallCommandPreview> {
    if skill.source_type != SourceType::Command {
        return None;
    }

    Some(InstallCommandPreview {
        skill_id: skill.id.clone(),
        skill_name: skill.name.clone(),
        command: skill.install_command.clone().unwrap_or_default(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn write_skill(path: &Path, name: &str) {
        fs::create_dir_all(path).unwrap();
        fs::write(
            path.join(SKILL_FILE_NAME),
            format!(
                "---\nname: {name}\ndescription: Test skill\nversion: 1.2.3\ntags:\n  - test\n---\n# {name}\n"
            ),
        )
        .unwrap();
    }

    #[test]
    fn parses_skill_frontmatter() {
        let temp = tempdir().unwrap();
        let skill_dir = temp.path().join("hello");
        write_skill(&skill_dir, "Hello Skill");

        let skill = parse_skill_folder(&skill_dir).unwrap();

        assert_eq!(skill.id, "hello-skill");
        assert_eq!(skill.name, "Hello Skill");
        assert_eq!(skill.description, "Test skill");
        assert_eq!(skill.version.as_deref(), Some("1.2.3"));
        assert_eq!(skill.tags, vec!["test"]);
    }

    #[test]
    fn default_data_dir_matches_tauri_app_identifier() {
        let data_dir = AppPaths::default_data_dir().unwrap();

        assert_eq!(data_dir.file_name().unwrap(), APP_IDENTIFIER);
    }

    #[test]
    fn discovers_nested_skill_folders() {
        let temp = tempdir().unwrap();
        write_skill(&temp.path().join("one"), "One");
        write_skill(&temp.path().join("nested").join("two"), "Two");

        let folders = discover_skill_folders(temp.path()).unwrap();

        assert_eq!(folders.len(), 2);
    }

    #[test]
    fn imports_by_copying_into_library() {
        let temp = tempdir().unwrap();
        let data_dir = temp.path().join("data");
        let source = temp.path().join("source");
        write_skill(&source, "Copy Me");

        let mut store = SkillsStore::open(&data_dir).unwrap();
        let imported = store.import_path(&source).unwrap();

        assert_eq!(imported.len(), 1);
        let library_path = imported[0].library_path.as_ref().unwrap();
        assert!(library_path.starts_with(data_dir.join("library").join("skills")));
        assert!(library_path.join(SKILL_FILE_NAME).exists());
    }

    #[test]
    fn creates_group_and_installs_group_into_agents_skills() {
        let temp = tempdir().unwrap();
        let data_dir = temp.path().join("data");
        let source = temp.path().join("source");
        let project = temp.path().join("project");
        fs::create_dir_all(&project).unwrap();
        write_skill(&source, "Install Me");

        let mut store = SkillsStore::open(&data_dir).unwrap();
        store.import_path(&source).unwrap();
        store.create_group("Starter".to_string(), None).unwrap();
        store.add_skill_to_group("starter", "install-me").unwrap();

        let outcome = store
            .install_group(
                "starter",
                Some(project.clone()),
                None,
                false,
                CommandExecutionMode::PreviewOnly,
            )
            .unwrap();

        assert_eq!(outcome.installed_skills.len(), 1);
        assert!(project
            .join(".agents")
            .join("skills")
            .join("install-me")
            .join(SKILL_FILE_NAME)
            .exists());
    }

    #[test]
    fn exports_local_skill() {
        let temp = tempdir().unwrap();
        let data_dir = temp.path().join("data");
        let source = temp.path().join("source");
        let export_dir = temp.path().join("export");
        write_skill(&source, "Export Me");

        let mut store = SkillsStore::open(&data_dir).unwrap();
        store.import_path(&source).unwrap();
        let exported = store.export_skill("export-me", &export_dir, false).unwrap();

        assert!(exported.join(SKILL_FILE_NAME).exists());
    }
}
