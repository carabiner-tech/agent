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
    timeout_duration: Duration,
) -> Result<CommandResult, std::io::Error> {
    // Setup the command to spawn
    let mut cmd = Command::new(command);
    cmd.args(args);
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
            child.kill().await?;
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

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;

    use tempfile::TempDir;

    use super::*;

    #[rstest::fixture]
    fn tmp_dir() -> TempDir {
        let dir = tempfile::tempdir().unwrap();
        std::env::set_current_dir(&dir).unwrap();
        dir
    }

    #[rstest::rstest]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_script_full_duration(_tmp_dir: TempDir) {
        let mut f = File::create("test.sh").unwrap();
        f.write_all(b"#!/bin/bash\necho 'Started'\nsleep 0.05\necho 'Finished'")
            .unwrap();
        let cmd = "bash";
        let args = vec!["test.sh"];
        let timeout_duration = Duration::from_secs(1);
        let CommandResult {
            stdout,
            stderr,
            exit_status,
        } = run_command_with_timeout(cmd, &args, timeout_duration)
            .await
            .unwrap();
        assert_eq!(stdout, "Started\nFinished\n");
        assert_eq!(stderr, "");
        assert_eq!(exit_status, Some(0));
    }

    #[rstest::rstest]
    #[tokio::test]
    #[serial_test::serial]
    async fn test_script_timeout(_tmp_dir: TempDir) {
        let mut f = File::create("test.sh").unwrap();
        f.write_all(b"#!/bin/bash\necho 'Started'\nsleep 0.05\necho 'Finished'")
            .unwrap();
        let cmd = "bash";
        let args = vec!["test.sh"];
        let timeout_duration = Duration::from_millis(10);
        let CommandResult {
            stdout,
            stderr,
            exit_status,
        } = run_command_with_timeout(cmd, &args, timeout_duration)
            .await
            .unwrap();
        assert_eq!(stdout, "Started\n");
        assert_eq!(stderr, "");
        assert_eq!(exit_status, None);
    }
}
