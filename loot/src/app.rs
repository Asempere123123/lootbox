use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::mpsc::{Receiver, Sender};
use toml;

use crate::commands::{Command, CommandOutput};

#[derive(Debug, Deserialize, Serialize)]
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

    pub async fn make_internal(&mut self, path: Option<std::path::PathBuf>) {
        let location = match path {
            Some(path) => path,
            None => std::path::PathBuf::new(),
        };

        if !location.join(crate::DEPENDENCIES_FILE).exists() {
            panic!("Not inside a project");
        } else if !location.join(".lootbox").exists() {
            // Se supone que si existe es valido
            println!("todo!, get python version here");
            Box::pin(crate::new::create_lootbox_dir(
                Some(&location),
                &"3.10.0".to_owned(),
                self,
            ))
            .await;
        }

        self.app_config = toml::from_str(
            &std::fs::read_to_string(location.join(crate::DEPENDENCIES_FILE))
                .expect("No config file. WTF"),
        )
        .expect("Error parsing config");

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

        self.sender
            .send(Command::InternalCommand(format!(
                ". {}",
                location.to_string_lossy().to_string()
            )))
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
