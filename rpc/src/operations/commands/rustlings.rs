use crate::operations::commands::utils::{run_command_with_timeout, CommandResult};
use crate::operations::fs::utils::ensure_relative;
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use std::{error::Error, path::PathBuf};
use tokio::time::Duration;

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct RustlingsVerifyRequest {}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct RustlingsVerifyResponse {
    pub stdout: String,
}

impl RustlingsVerifyRequest {
    pub async fn process(self) -> Result<RustlingsVerifyResponse, Box<dyn Error>> {
        let cmd = "rustlings";
        let args = vec!["verify"];
        let timeout_duration = Duration::from_secs(5);
        let CommandResult {
            stdout,
            stderr,
            exit_status,
        } = run_command_with_timeout(cmd, &args, timeout_duration).await?;
        Ok(RustlingsVerifyResponse { stdout })
    }
}
