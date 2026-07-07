use anyhow::{bail, Result};
use clap::{Args, Parser, Subcommand};
use skills_core::{CommandExecutionMode, InstallationOutcome, Skill, SkillsStore};
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "skills-list")]
#[command(version, about = "Catalog, search, group, and install Codex skills.")]
struct Cli {
    #[arg(long, value_name = "DIR", global = true, env = "SKILLS_LIST_DATA_DIR")]
    data_dir: Option<PathBuf>,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Search(SearchArgs),
    Import(PathArg),
    AddCommand(AddCommandArgs),
    Group {
        #[command(subcommand)]
        command: GroupCommands,
    },
    Install {
        #[command(subcommand)]
        command: InstallCommands,
    },
}

#[derive(Args)]
struct SearchArgs {
    query: String,
    #[arg(long)]
    json: bool,
}

#[derive(Args)]
struct PathArg {
    path: PathBuf,
}

#[derive(Args)]
struct AddCommandArgs {
    name: String,
    #[arg(long)]
    description: String,
    #[arg(long = "command")]
    install_command: String,
    #[arg(long = "tag")]
    tags: Vec<String>,
}

#[derive(Subcommand)]
enum GroupCommands {
    Create(GroupCreateArgs),
    Add(GroupAddArgs),
    Remove(GroupAddArgs),
    Delete(GroupDeleteArgs),
}

#[derive(Args)]
struct GroupCreateArgs {
    name: String,
    #[arg(long)]
    description: Option<String>,
}

#[derive(Args)]
struct GroupAddArgs {
    group: String,
    skill: String,
}

#[derive(Args)]
struct GroupDeleteArgs {
    group: String,
}

#[derive(Subcommand)]
enum InstallCommands {
    Skill(InstallTargetArgs),
    Group(InstallTargetArgs),
}

#[derive(Args)]
struct InstallTargetArgs {
    reference: String,
    #[arg(long)]
    project: Option<PathBuf>,
    #[arg(long)]
    target: Option<PathBuf>,
    #[arg(long)]
    overwrite: bool,
    #[arg(long)]
    yes: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut store = match cli.data_dir {
        Some(data_dir) => SkillsStore::open(data_dir)?,
        None => SkillsStore::open_default()?,
    };

    match cli.command {
        Commands::Search(args) => search(&store, args),
        Commands::Import(args) => import(&mut store, args),
        Commands::AddCommand(args) => add_command(&mut store, args),
        Commands::Group { command } => group(&mut store, command),
        Commands::Install { command } => install(&store, command),
    }
}

fn search(store: &SkillsStore, args: SearchArgs) -> Result<()> {
    let skills = store.search(&args.query);
    if args.json {
        println!("{}", serde_json::to_string_pretty(&skills)?);
        return Ok(());
    }

    if skills.is_empty() {
        println!("No skills found.");
        return Ok(());
    }

    for skill in skills {
        print_skill_line(&skill);
    }
    Ok(())
}

fn import(store: &mut SkillsStore, args: PathArg) -> Result<()> {
    let imported = store.import_path(args.path)?;
    for skill in imported {
        println!("Imported {} ({})", skill.name, skill.id);
    }
    Ok(())
}

fn add_command(store: &mut SkillsStore, args: AddCommandArgs) -> Result<()> {
    let skill =
        store.add_command_skill(args.name, args.description, args.install_command, args.tags)?;
    println!("Added command skill {} ({})", skill.name, skill.id);
    Ok(())
}

fn group(store: &mut SkillsStore, command: GroupCommands) -> Result<()> {
    match command {
        GroupCommands::Create(args) => {
            let group = store.create_group(args.name, args.description)?;
            println!("Created group {} ({})", group.name, group.id);
        }
        GroupCommands::Add(args) => {
            let group = store.add_skill_to_group(&args.group, &args.skill)?;
            println!("Added {} to group {}", args.skill, group.name);
        }
        GroupCommands::Remove(args) => {
            let group = store.remove_skill_from_group(&args.group, &args.skill)?;
            println!("Removed {} from group {}", args.skill, group.name);
        }
        GroupCommands::Delete(args) => {
            let group = store.delete_group(&args.group)?;
            println!("Deleted group {} ({})", group.name, group.id);
        }
    }
    Ok(())
}

fn install(store: &SkillsStore, command: InstallCommands) -> Result<()> {
    match command {
        InstallCommands::Skill(args) => {
            let command_mode = command_execution_mode(store, true, &args)?;
            let outcome = store.install_skill(
                &args.reference,
                args.project,
                args.target,
                args.overwrite,
                command_mode,
            )?;
            print_install_outcome(outcome);
        }
        InstallCommands::Group(args) => {
            let command_mode = command_execution_mode(store, false, &args)?;
            let outcome = store.install_group(
                &args.reference,
                args.project,
                args.target,
                args.overwrite,
                command_mode,
            )?;
            print_install_outcome(outcome);
        }
    }
    Ok(())
}

fn command_execution_mode(
    store: &SkillsStore,
    single_skill: bool,
    args: &InstallTargetArgs,
) -> Result<CommandExecutionMode> {
    let previews = if single_skill {
        store.preview_skill_commands(&args.reference)?
    } else {
        store.preview_group_commands(&args.reference)?
    };

    if previews.is_empty() {
        return Ok(CommandExecutionMode::PreviewOnly);
    }

    for command in &previews {
        println!("Install command for {}:", command.skill_name);
        println!("{}", command.command);
    }

    if args.yes {
        return Ok(CommandExecutionMode::InteractiveTerminal);
    }

    print!("Run these command skills now? [y/N] ");
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    if matches!(answer.trim().to_lowercase().as_str(), "y" | "yes") {
        Ok(CommandExecutionMode::InteractiveTerminal)
    } else {
        bail!("installation cancelled before running command skills")
    }
}

fn print_install_outcome(outcome: InstallationOutcome) {
    println!("Target: {}", outcome.target_dir.display());
    for installed in outcome.installed_skills {
        if let Some(target) = installed.target_path {
            println!("Installed {} -> {}", installed.skill_name, target.display());
        } else if installed.executed {
            println!("Executed command skill {}", installed.skill_name);
        }
    }
    for preview in outcome.command_previews {
        println!(
            "Pending command for {}: {}",
            preview.skill_name, preview.command
        );
    }
}

fn print_skill_line(skill: &Skill) {
    let version = skill.version.as_deref().unwrap_or("no-version");
    println!("{} [{}] - {}", skill.name, version, skill.description);
    println!("  id: {}", skill.id);
}
