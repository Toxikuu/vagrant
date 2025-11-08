// utils/sh.rs

use std::collections::HashMap;
use std::process::{Command, Stdio};

use color_eyre::Result;
use color_eyre::eyre::Context;
use thiserror::Error;
use tracing::{trace, warn};

#[derive(Error, Debug)]
enum CmdError {
    #[error("output in stderr")]
    OutputInStderr,

    #[error("nonzero status")]
    NonzeroStatus,

    #[error("empty stdout")]
    EmptyStdout,
}

/// # Lowish level function to execute a command and return stdout
#[allow(clippy::similar_names)]
pub fn cmd(cmd: &[&str], env: HashMap<&str, &str>, cwd: &str) -> Result<String> {
    trace!("Evaluating command: {}", cmd.join(" "));

    let (arg0, args) = cmd.split_first().expect("command should not be empty");
    let child = Command::new(arg0)
        .args(args)
        .envs(env)
        .current_dir(cwd)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .wrap_err("Failed to spawn command")?;

    let output = child
        .wait_with_output()
        .wrap_err("Failed to wait on child")?;
    let code = output.status.code().unwrap_or(1);
    let out = String::from_utf8_lossy(&output.stdout).to_string();
    let err = String::from_utf8_lossy(&output.stderr).to_string();

    trace!("STDOUT: {out}");

    if !err.is_empty() {
        warn!("STDERR: {err}");
        return Err(CmdError::OutputInStderr).wrap_err("Output in stderr");
    }

    if code != 0 {
        warn!("Exited with nonzero status: {code}");
        return Err(CmdError::NonzeroStatus).wrap_err("Exited with nonzero status");
    }

    if out.trim().is_empty() {
        warn!("No output in stdout");
        return Err(CmdError::EmptyStdout).wrap_err("No output in stdout");
    }

    Ok(out)
}
