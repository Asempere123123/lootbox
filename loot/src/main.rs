use clap::{Parser, Subcommand};
use directories::ProjectDirs;
use inline_colorization::*;

mod config;
mod install;
mod new;
mod utils;

const DEPENDENCIES_FILE: &str = "requirements.toml";
const PYTHON_INSTALLS_DIRECTORY: &str = "python_installs";

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Turn debugging information on
    #[arg(short, long, action = clap::ArgAction::SetTrue)]
    debug: bool,

    /// Activate dev mode
    #[arg(long, action = clap::ArgAction::SetTrue)]
    dev: bool,

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
}

fn main() {
    let cli = Cli::parse();

    let project_dirs =
        ProjectDirs::from("cli", "Asempere", "py-lootbox").expect("Project dir not found");
    let data_path = project_dirs.data_dir();

    if cli.debug {
        println!("{color_yellow}Debug mode is on{color_reset}");
    }

    if cli.dev {
        println!("{color_cyan}Dev mode is {color_yellow}ACTIVE{color_reset}");

        let _ = std::fs::remove_dir_all("../test");
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

            new::new_project(&cli, data_path, name, python_version);
        }
        Some(Commands::Install { version, force }) => {
            install::install_version(&cli, data_path, version, force);
        }
        None => {}
    }
}
