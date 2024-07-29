use std::process;
use tokio::sync::mpsc::Sender;

pub enum Command {
    InternalCommand(String, Option<String>),
    InternalCommandWithOutput(String, Option<String>),
    ExternalCommand(String),
    ExternalCommandWithOutput(String),
}

pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
}

pub async fn execute_command(cmd: Command, sender: &Sender<CommandOutput>) {
    match cmd {
        Command::ExternalCommand(cmd) => {
            let command_parts: Vec<&str> = cmd.split_whitespace().collect();

            let _ = process::Command::new(command_parts[0])
                .args(&command_parts[1..])
                .stdout(process::Stdio::inherit())
                .stderr(process::Stdio::inherit())
                .output()
                .expect("Error running external command");
        }
        Command::ExternalCommandWithOutput(cmd) => {
            let command_parts: Vec<&str> = cmd.split_whitespace().collect();

            let result = process::Command::new(command_parts[0])
                .args(&command_parts[1..])
                .output()
                .expect("Error running external command");

            sender
                .send(CommandOutput {
                    stdout: String::from_utf8_lossy(&result.stdout).to_string(),
                    stderr: String::from_utf8_lossy(&result.stderr).to_string(),
                })
                .await
                .expect("channel was closed. WTF");
        }
        Command::InternalCommand(cmd, path) => todo!(),
        Command::InternalCommandWithOutput(cmd, path) => todo!(),
    }
}
