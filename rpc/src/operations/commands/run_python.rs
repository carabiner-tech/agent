use crate::operations::commands::utils::{run_command_with_timeout, CommandResult};
use crate::operations::fs::utils::ensure_relative;
use poem_openapi::Object;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, error::Error, path::PathBuf};
use tokio::time::Duration;

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct RunPythonRequest {
    pub path: String,
}

#[derive(Debug, Serialize, Deserialize, Object)]
pub struct RunPythonResponse {
    pub stdout: String,
    pub stderr: String,
    pub exit_status: Option<i32>,
}

impl RunPythonRequest {
    pub async fn process(self) -> Result<RunPythonResponse, Box<dyn Error>> {
        let path = ensure_relative(PathBuf::from(self.path)).await?;
        let cmd = "python";
        let args = vec![path.to_str().unwrap()];
        let mut env_vars = HashMap::new();
        env_vars.insert("PYTHONUNBUFFERED".to_string(), "1".to_string());
        let timeout_duration = Duration::from_secs(5);
        let CommandResult {
            stdout,
            stderr,
            exit_status,
        } = run_command_with_timeout(cmd, &args, env_vars, timeout_duration).await?;
        Ok(RunPythonResponse {
            stdout,
            stderr,
            exit_status,
        })
    }
}
