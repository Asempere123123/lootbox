use tokio::sync::mpsc::{Receiver, Sender};

use crate::commands::{Command, CommandOutput};

pub struct AppExternal<'a> {
    pub data_path: &'a std::path::Path,

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

        #[cfg(target_os = "windows")]
        let location = location
            .join(".lootbox")
            .join("venv")
            .join("Scripts")
            .join("Activate.ps1");

        self.sender
            .send(Command::InternalCommand(
                location.to_string_lossy().to_string(),
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
