use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use inline_colorization::*;
use std::path::PathBuf;
use tokio;

mod add;
mod app;
mod commands;
mod install;
mod new;
mod python_dependency_resolver;
mod run;
mod utils;
mod versions;

use crate::install::install_python_version;
use add::add_dependency;
use app::AppExternal;
use commands::execute_command;
use new::new_project;
use run::run_app;

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

        /// If active will remove contents inside target dir
        #[arg(short, long, action = clap::ArgAction::SetTrue)]
        force: bool,
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

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let cli = Cli::parse();

    let project_dirs =
        ProjectDirs::from("cli", "Asempere", "py-lootbox").expect("Project dir not found");
    let data_path = project_dirs.data_dir();

    let (sender, mut receiver) = tokio::sync::mpsc::channel(5);
    let (response_sender, response_receiver) = tokio::sync::mpsc::channel(1);
    let commands_thread_handle = tokio::spawn(async move {
        use std::io::Write;

        #[cfg(target_os = "windows")]
        let mut child = std::process::Command::new("powershell")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Error creating child process");

        #[cfg(not(target_os = "windows"))]
        let mut child = std::process::Command::new("sh")
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn()
            .expect("Error creating child process");

        while let Some(command) = receiver.recv().await {
            execute_command(command, &mut child, &response_sender).await;
        }

        writeln!(child.stdin.as_mut().unwrap(), "exit").expect("Error writting to stdin");
        let _ = child.wait();
    });

    let mut app = AppExternal::new(data_path, sender, response_receiver);

    if cli.debug {
        println!("{color_yellow}Debug mode is on{color_reset}");
    }

    match &cli.command {
        Some(Commands::New {
            name,
            python_version,
            force,
        }) => {
            println!(
                r#"Creating project with name "{color_yellow}{}{color_reset}""#,
                name.file_name()
                    .expect("No name selected")
                    .to_string_lossy()
            );

            new_project(name, python_version, force, app).await;
        }
        Some(Commands::Install { version, force }) => {
            install_python_version(version, force, app).await;
        }
        Some(Commands::Run) => {
            run_app(app).await;
        }
        Some(Commands::Add { package, version }) => {
            add_dependency(package, version, app).await;
        }
        Some(Commands::Exec { command }) => {
            app.make_internal(None).await;
            app.run_internal_command(command.join(" ")).await;
            drop(app);
        }
        Some(Commands::Bundle) => {
            println!("{color_yellow}Remember to run the project once at least before bundling to resolve its dependencies{color_reset}");
            let _ = std::fs::remove_dir_all(&PathBuf::from("./target"));
            crate::utils::clone_dir(&PathBuf::from("./src"), &PathBuf::from("./target"))
                .expect("Error cloning souce code");
            app.make_internal(None).await;
            app.run_internal_command("pip freeze > ./target/requirements.txt".to_owned())
                .await;
            drop(app);
        }
        None => {
            drop(app);
            println!(
                "py-lootbox {}, type 'loot help' for info",
                env!("CARGO_PKG_VERSION")
            )
        }
    };

    while !commands_thread_handle.is_finished() {}
}
