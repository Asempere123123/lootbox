use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use inline_colorization::*;
use tokio;

mod app;
mod commands;
mod install;

use crate::install::install_python_version;
use app::AppExternal;
use commands::execute_command;

const DEPENDENCIES_FILE: &str = "lootbox.toml";
const PYTHON_INSTALLS_DIRECTORY: &str = "python_installs";

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    debug: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Creates a new project
    New {
        /// Name of the project
        name: std::path::PathBuf,

        /// Version of python to use
        python_version: String,
    },
    /// Installs a new python version
    Install {
        /// Version to install
        version: String,

        /// If active will override previous installation
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        force: bool,
    },
    /// Runs python project
    Run,
    /// Adds a dependency for the current project
    Add {
        /// Package to add
        package: String,

        /// Version to add
        #[arg(short, long)]
        version: Option<String>,
    },
    /// Runs a command inside the venv. Usefull if using dependencies that have a Cli
    Exec {
        /// Command to run
        #[arg(allow_hyphen_values = true)]
        command: Vec<String>,
    },
    /// Bundle the project into a version executable without lootbox
    Bundle,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let project_dirs =
        ProjectDirs::from("cli", "Asempere", "py-lootbox").expect("Project dir not found");
    let data_path = project_dirs.data_dir();

    let (sender, mut receiver) = tokio::sync::mpsc::channel(5);
    let (response_sender, mut response_receiver) = tokio::sync::mpsc::channel(1);
    let commands_thread_handle = tokio::spawn(async move {
        while let Some(command) = receiver.recv().await {
            execute_command(command, &response_sender).await;
        }
    });

    let app = AppExternal::new(data_path, sender, response_receiver);

    if cli.debug {
        println!("{color_yellow}Debug mode is on{color_reset}");
    }

    match &cli.command {
        Some(Commands::New {
            name,
            python_version,
        }) => {
            println!(
                r#"Creating project with name "{color_yellow}{}{color_reset}""#,
                name.file_name()
                    .expect("No name selected")
                    .to_string_lossy()
            );

            todo!();
        }
        Some(Commands::Install { version, force }) => {
            install_python_version(version, force, app).await;
        }
        Some(Commands::Run) => {
            todo!();
        }
        Some(Commands::Add { package, version }) => {
            todo!();
        }
        Some(Commands::Exec { command }) => {
            todo!();
        }
        Some(Commands::Bundle) => {
            todo!();
        }
        None => println!(
            "py-lootbox {}, type 'loot help' for info",
            env!("CARGO_PKG_VERSION")
        ),
    };

    while !commands_thread_handle.is_finished() {}
}
