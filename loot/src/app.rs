use tokio::sync::mpsc::{Receiver, Sender};

use crate::commands::{Command, CommandOutput};

pub struct AppExternal<'a> {
    pub data_path: &'a std::path::Path,

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

    pub fn make_internal(self) -> AppInternal {
        // Add verifications to check if this is possible
        println!("make_internal is unsafe, it is needed to add verifications to check wether user is inside a lootbox context");

        AppInternal {
            sender: self.sender,
            receiver: self.receiver,
        }
    }
}

pub struct AppInternal {
    sender: Sender<Command>,
    receiver: Receiver<CommandOutput>,
}

impl AppInternal {
    pub async fn run_internal_command(&self, command: String) {
        self.sender
            .send(Command::InternalCommand(command, None))
            .await
            .expect(
                "Command receiver droped, this should NEVER happen, the commands thread crashed.",
            );
    }

    pub async fn run_internal_command_with_output(&mut self, command: String) -> CommandOutput {
        self.sender
            .send(Command::InternalCommandWithOutput(command, None))
            .await
            .expect(
                "Command receiver droped, this should NEVER happen, the commands thread crashed.",
            );

        self.receiver.recv().await.expect("Command output receiver droped, this should NEVER happen, the commands thread crashed.")
    }
}
