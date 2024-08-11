use std::io::{BufRead, BufReader, Read, Write};
use std::ops::DerefMut;
use std::path::PathBuf;
use std::process;
use std::sync::{Arc, Mutex};
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
    stdout: Arc<Mutex<std::process::ChildStdout>>,
    stderr: Arc<Mutex<std::process::ChildStderr>>,
    stdin: Arc<tokio::sync::Mutex<std::process::ChildStdin>>,
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
            let mut stdin = stdin.lock().await;

            #[cfg(target_os = "windows")]
            writeln!(
                stdin,
                "{}; echo finalizau; $host.ui.WriteErrorLine('finalizau')",
                cmd
            )
            .expect("Error writing to stdin");

            #[cfg(not(target_os = "windows"))]
            writeln!(stdin, "{}; echo finalizau; echo finalizau 1>&2", cmd)
                .expect("Error writing to stdin");

            drop(stdin);

            let stdout_handle = tokio::spawn(async move {
                let mut stdout = stdout.lock().expect("Lock got poisoned");

                let mut buffer = [0; 1024];
                loop {
                    let bytes_read = stdout.read(&mut buffer).expect("Error reading stdout");

                    // Sino da error por overflow si el input es menor que finalizau
                    if bytes_read >= b"finalizau".len() {
                        if let Some(finalizau_location) =
                            bytes_contains_subytes(&buffer[..bytes_read], b"finalizau")
                        {
                            print!("{}", String::from_utf8_lossy(&buffer[..finalizau_location]));
                            break;
                        }
                    }

                    print!("{}", String::from_utf8_lossy(&buffer[..bytes_read]));
                }
            });

            let stderr_handle = tokio::spawn(async move {
                let mut stderr = stderr.lock().expect("Lock got poisoned");
                let stderr_wrapper = BufReader::new(stderr.deref_mut());

                for line in stderr_wrapper.lines() {
                    let line = line.unwrap_or_default();

                    if line == "finalizau" {
                        break;
                    }

                    println!("{}", line);
                }
            });

            let _ = tokio::join!(stdout_handle, stderr_handle);
        }
        Command::InternalCommandWithOutput(cmd) => {
            let mut stdin = stdin.lock().await;

            #[cfg(target_os = "windows")]
            writeln!(
                stdin,
                "{}; echo finalizau; $host.ui.WriteErrorLine('finalizau')",
                cmd
            )
            .expect("Error writing to stdin");

            #[cfg(not(target_os = "windows"))]
            writeln!(stdin, "{}; echo finalizau; echo finalizau 1>&2", cmd)
                .expect("Error writing to stdin");

            drop(stdin);

            let stdout_handle = tokio::spawn(async move {
                let mut stdout_string = String::new();

                let mut stdout = stdout.lock().expect("Lock got poisoned");
                let stdout_wrapper = BufReader::new(stdout.deref_mut());

                for line in stdout_wrapper.lines() {
                    let line = line.unwrap_or_default();

                    if line == "finalizau" {
                        break;
                    }

                    stdout_string += &(line + "\n");
                }
                stdout_string
            });

            let stderr_handle = tokio::spawn(async move {
                let mut stderr_string = String::new();

                let mut stderr = stderr.lock().expect("Lock got poisoned");
                let stderr_wrapper = BufReader::new(stderr.deref_mut());

                for line in stderr_wrapper.lines() {
                    let line = line.unwrap_or_default();

                    if line == "finalizau" {
                        break;
                    }

                    stderr_string += &(line + "\n");
                }
                stderr_string
            });

            let (stdout_string, stderr_string) = tokio::join!(stdout_handle, stderr_handle);

            sender
                .send(CommandOutput {
                    stdout: stdout_string.expect("Error handling stdout"),
                    stderr: stderr_string.expect("Error handling stderr"),
                })
                .await
                .expect("channel was closed. WTF");
        }
    }
}

fn bytes_contains_subytes(haystack: &[u8], needle: &[u8]) -> Option<usize> {
    (0..haystack.len() - needle.len() + 1)
        .filter(|&i| haystack[i..i + needle.len()] == needle[..])
        .next()
}
