use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process;
use tokio::sync::mpsc::Sender;

#[derive(Debug)]
pub enum Command {
    InternalCommand(String),
    InternalCommandWithOutput(String),
    ExternalCommand(String),
    ExternalCommandWithOutput(String),
    ExternalCommandFromDirectory(String, PathBuf),
}

#[derive(Debug)]
#[allow(unused)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: String,
}

pub async fn execute_command(
    cmd: Command,
    child: &mut std::process::Child,
    sender: &Sender<CommandOutput>,
) {
    println!("{:?}", cmd);
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
        Command::ExternalCommandFromDirectory(cmd, dir) => {
            let command_parts: Vec<&str> = cmd.split_whitespace().collect();

            let _ = process::Command::new(command_parts[0])
                .current_dir(dir)
                .args(&command_parts[1..])
                .stdout(process::Stdio::inherit())
                .stderr(process::Stdio::inherit())
                .output()
                .expect("Error running external command");
        }
        Command::InternalCommand(cmd) => {
            let stdin = child.stdin.as_mut().expect("Stdin does not exist");

            #[cfg(target_os = "windows")]
            writeln!(
                stdin,
                "{}; echo finalizau; $host.ui.WriteErrorLine('finalizau')",
                cmd
            )
            .expect("Error writing to stdin");

            #[cfg(not(target_os = "windows"))]
            writeln!(stdin, "{} && echo finalizau && echo finalizau 1>&2", cmd)
                .expect("Error writing to stdin");

            let stdout = BufReader::new(child.stdout.as_mut().expect("Stdout does not exist"));
            let stderr = BufReader::new(child.stderr.as_mut().expect("Stderr does not exist"));

            for line in stdout.lines() {
                let line = line.unwrap_or_default();

                if line == "finalizau" {
                    break;
                }

                println!("{}", line);
            }

            for line in stderr.lines() {
                let line = line.unwrap_or_default();

                if line.ends_with("finalizau") {
                    break;
                }

                println!("{}", line);
            }
        }
        Command::InternalCommandWithOutput(cmd) => {
            let stdin = child.stdin.as_mut().expect("Stdin does not exist");

            #[cfg(target_os = "windows")]
            writeln!(
                stdin,
                "{}; echo finalizau; $host.ui.WriteErrorLine('finalizau')",
                cmd
            )
            .expect("Error writing to stdin");

            #[cfg(not(target_os = "windows"))]
            writeln!(stdin, "{} && echo finalizau && echo finalizau 1>&2", cmd)
                .expect("Error writing to stdin");

            let stdout = BufReader::new(child.stdout.as_mut().expect("Stdout does not exist"));
            let stderr = BufReader::new(child.stderr.as_mut().expect("Stderr does not exist"));

            let mut stdout_string = String::new();
            let mut stderr_string = String::new();
            for line in stdout.lines() {
                let line = line.unwrap_or_default();

                if line == "finalizau" {
                    break;
                }

                stdout_string += &(line + "\n");
            }

            for line in stderr.lines() {
                let line = line.unwrap_or_default();

                if line.ends_with("finalizau") {
                    break;
                }

                stderr_string += &(line + "\n");
            }

            sender
                .send(CommandOutput {
                    stdout: stdout_string,
                    stderr: stderr_string,
                })
                .await
                .expect("channel was closed. WTF");
        }
    }
}
