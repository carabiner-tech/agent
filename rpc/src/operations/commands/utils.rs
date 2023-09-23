use std::collections::HashMap;

use std::process::Stdio;
use tokio::io::{AsyncRead, AsyncReadExt};
use tokio::process::Command;
use tokio::time::{timeout, Duration};

#[derive(Debug)]
pub struct CommandResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_status: Option<i32>, // None if the process was killed due to timeout
}

// Used for reading stdout / stderr from process, even if process is killed due to timeout
// (normally .read_to_end is used but that won't work if process is killed)
async fn read_stream<R: AsyncRead + Unpin>(mut reader: R) -> Vec<u8> {
    let mut buffer = Vec::new();
    let mut chunk = [0; 1024];
    while let Ok(size) = reader.read(&mut chunk).await {
        if size == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..size]);
    }
    buffer
}

pub async fn run_command_with_timeout(
    command: &str,
    args: &[&str],
    env_vars: HashMap<String, String>,
    timeout_duration: Duration,
) -> Result<CommandResult, std::io::Error> {
    // Setup the command to spawn
    let mut cmd = Command::new(command);
    cmd.args(args);
    cmd.envs(env_vars);
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd.kill_on_drop(true);

    let mut child = cmd.spawn()?;

    let stdout_handle = tokio::spawn(read_stream(child.stdout.take().unwrap()));
    let stderr_handle = tokio::spawn(read_stream(child.stderr.take().unwrap()));

    match timeout(timeout_duration, child.wait()).await {
        Ok(exit_status) => {
            let stdout = String::from_utf8(stdout_handle.await.unwrap()).unwrap();
            let stderr = String::from_utf8(stderr_handle.await.unwrap()).unwrap();
            Ok(CommandResult {
                stdout,
                stderr,
                exit_status: Some(exit_status.unwrap().code().unwrap()),
            })
        }
        Err(_) => {
            let stdout = String::from_utf8(stdout_handle.await.unwrap()).unwrap();
            let stderr = String::from_utf8(stderr_handle.await.unwrap()).unwrap();
            Ok(CommandResult {
                stdout,
                stderr,
                exit_status: None,
            })
        }
    }
}
