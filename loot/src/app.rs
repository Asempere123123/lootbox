use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process;
use tokio::{
    sync::mpsc::{Receiver, Sender},
    task::JoinHandle,
};
use toml;

use crate::commands::{Command, CommandOutput};

#[derive(Debug, Deserialize, Serialize, PartialEq, Eq, Clone)]
pub struct Config {
    pub name: String,
    pub python_version: String,
    pub requirements: HashMap<String, String>,
}

pub struct AppExternal<'a> {
    pub data_path: &'a std::path::Path,
    pub app_config: Option<Config>,

    is_internal: bool,
    sender: Sender<Command>,
    receiver: Receiver<CommandOutput>,
}

impl<'a> AppExternal<'a> {
    pub fn new(
        data_path: &'a std::path::Path,
        sender: Sender<Command>,
        receiver: Receiver<CommandOutput>,
    ) -> Self {
        Self {
            data_path,
            app_config: None,
            is_internal: false,
            sender,
            receiver,
        }
    }

    pub async fn run_external_command(&self, command: String) {
        self.sender
            .send(Command::ExternalCommand(command))
            .await
            .expect(
                "Command receiver droped, this should NEVER happen, the commands thread crashed.",
            );
    }

    pub async fn run_external_command_from_dir(&self, command: String, dir: PathBuf) {
        self.sender
            .send(Command::ExternalCommandFromDirectory(command, dir))
            .await
            .expect(
                "Command receiver droped, this should NEVER happen, the commands thread crashed.",
            );
    }

    pub async fn run_external_command_with_output(&mut self, command: String) -> CommandOutput {
        self.sender
            .send(Command::ExternalCommandWithOutput(command))
            .await
            .expect(
                "Command receiver droped, this should NEVER happen, the commands thread crashed.",
            );

        self.receiver.recv().await.expect("Command output receiver droped, this should NEVER happen, the commands thread crashed.")
    }

    #[cfg(target_os = "windows")]
    pub fn get_python_binary(&self, python_version: &String) -> Option<std::path::PathBuf> {
        let path = self
            .data_path
            .join(crate::PYTHON_INSTALLS_DIRECTORY)
            .join(python_version)
            .join(format!("python.{}", python_version))
            .join("tools")
            .join("python.exe");

        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    #[cfg(not(target_os = "windows"))]
    pub fn get_python_binary(&self, python_version: &String) -> Option<std::path::PathBuf> {
        let path = self
            .data_path
            .join(crate::PYTHON_INSTALLS_DIRECTORY)
            .join(python_version)
            .join("bin")
            .join("python3");

        if path.exists() {
            Some(path)
        } else {
            None
        }
    }

    pub fn get_old_config(path: Option<std::path::PathBuf>) -> Config {
        let location = match path {
            Some(path) => path,
            None => std::path::PathBuf::new(),
        };

        let path_to_file = location.join(".lootbox").join(crate::DEPENDENCIES_FILE);

        let old_config = std::fs::read_to_string(path_to_file).expect("Error reading old config");

        toml::from_str(&old_config).expect("Error parsing old config")
    }

    // This function does not check wether in valid context
    pub async fn get_access_venv_command(&self, path: Option<std::path::PathBuf>) -> String {
        let location = match path {
            Some(path) => path,
            None => std::path::PathBuf::new(),
        };

        #[cfg(target_os = "windows")]
        let location = location
            .join(".lootbox")
            .join("venv")
            .join("Scripts")
            .join("Activate.ps1");

        #[cfg(not(target_os = "windows"))]
        let location = location
            .join(".lootbox")
            .join("venv")
            .join("bin")
            .join("activate");

        #[cfg(target_os = "windows")]
        return location.to_string_lossy().to_string();

        #[cfg(not(target_os = "windows"))]
        return format!(". {}", location.to_string_lossy().to_string());
    }

    pub async fn run_paralel_internal_command(
        &self,
        path: Option<std::path::PathBuf>,
        command: String,
    ) -> JoinHandle<()> {
        #[cfg(target_os = "windows")]
        let command_to_run = format!(
            "{}; {}",
            self.get_access_venv_command(path).await.clone(),
            command
        );

        #[cfg(not(target_os = "windows"))]
        let command_to_run = format!(
            ". {} && {}",
            self.get_access_venv_command(path).await.clone(),
            command
        );

        tokio::spawn(async move {
            #[cfg(target_os = "windows")]
            let _ = process::Command::new("powershell")
                .args(&["-Command", &command_to_run])
                .stdout(process::Stdio::inherit())
                .stderr(process::Stdio::inherit())
                .output()
                .expect("Error running command");

            #[cfg(not(target_os = "windows"))]
            let _ = process::Command::new("sh")
                .args(&["-c", &command_to_run])
                .stdout(process::Stdio::inherit())
                .stderr(process::Stdio::inherit())
                .output()
                .expect("Error running command");
        })
    }

    pub async fn make_internal(&mut self, path: Option<std::path::PathBuf>) {
        let location = match path {
            Some(path) => path,
            None => std::path::PathBuf::new(),
        };

        self.app_config = toml::from_str(
            &std::fs::read_to_string(location.join(crate::DEPENDENCIES_FILE))
                .expect("No config file. Might not be inside a lootbox context"),
        )
        .expect("Error parsing config");

        // Validation
        if !location.join(crate::DEPENDENCIES_FILE).exists() {
            panic!("Not inside a project");
        } else if !location.join(".lootbox").exists() {
            // Se supone que si existe es valido
            println!("todo!, get python version here");
            Box::pin(crate::new::create_lootbox_dir(
                Some(&location),
                &self.app_config.clone().unwrap().python_version,
                self,
            ))
            .await;
        }

        self.sender
            .send(Command::InternalCommand(
                self.get_access_venv_command(Some(location)).await,
            ))
            .await
            .expect(
                "Command receiver droped, this should NEVER happen, the commands thread crashed.",
            );

        self.is_internal = true;
    }

    pub async fn run_internal_command(&self, command: String) -> Option<()> {
        if !self.is_internal {
            return None;
        }

        self.sender
            .send(Command::InternalCommand(command))
            .await
            .expect(
                "Command receiver droped, this should NEVER happen, the commands thread crashed.",
            );

        Some(())
    }

    pub async fn run_internal_command_with_output(
        &mut self,
        command: String,
    ) -> Option<CommandOutput> {
        if !self.is_internal {
            return None;
        }

        self.sender
            .send(Command::InternalCommandWithOutput(command))
            .await
            .expect(
                "Command receiver droped, this should NEVER happen, the commands thread crashed.",
            );

        Some(self.receiver.recv().await.expect("Command output receiver droped, this should NEVER happen, the commands thread crashed."))
    }
}
