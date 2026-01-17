//! Helpers for running child processes with timeouts and bounded output.

use std::io::{Read, Write};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use wait_timeout::ChildExt;

/// Captured child process output.
#[derive(Debug)]
pub struct CommandOutput {
    pub status: ExitStatus,
    pub stdout: Vec<u8>,
    pub stderr: Vec<u8>,
    pub stdout_truncated: usize,
    pub stderr_truncated: usize,
    pub timed_out: bool,
}

impl CommandOutput {
    pub fn stdout_truncated_notice(&self, label: &str) -> String {
        if self.stdout_truncated > 0 {
            format!(
                "\n[{label} stdout truncated {} bytes]\n",
                self.stdout_truncated
            )
        } else {
            String::new()
        }
    }

    pub fn stderr_truncated_notice(&self, label: &str) -> String {
        if self.stderr_truncated > 0 {
            format!(
                "\n[{label} stderr truncated {} bytes]\n",
                self.stderr_truncated
            )
        } else {
            String::new()
        }
    }
}

/// Run a command with a timeout and capture stdout/stderr without risking pipe deadlocks.
///
/// Output is read concurrently while the child runs. `output_limit_bytes` bounds the amount of
/// stdout/stderr stored in memory (bytes beyond this are discarded while still draining the pipe).
pub fn run_command_with_timeout(
    mut cmd: Command,
    stdin: Option<&[u8]>,
    timeout: Duration,
    output_limit_bytes: usize,
) -> Result<CommandOutput> {
    if stdin.is_some() {
        cmd.stdin(Stdio::piped());
    } else {
        cmd.stdin(Stdio::null());
    }
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    let mut child = cmd.spawn().context("spawn command")?;

    if let Some(input) = stdin {
        let mut child_stdin = child
            .stdin
            .take()
            .ok_or_else(|| anyhow!("stdin was not piped"))?;
        child_stdin.write_all(input).context("write stdin")?;
    }

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("stdout was not piped"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow!("stderr was not piped"))?;

    let stdout_handle = thread::spawn(move || read_stream_limited(stdout, output_limit_bytes));
    let stderr_handle = thread::spawn(move || read_stream_limited(stderr, output_limit_bytes));

    let mut timed_out = false;
    let status = match child.wait_timeout(timeout).context("wait for command")? {
        Some(status) => status,
        None => {
            timed_out = true;
            child.kill().context("kill command")?;
            child.wait().context("wait command after kill")?
        }
    };

    let (stdout, stdout_truncated) = join_output(stdout_handle).context("join stdout")?;
    let (stderr, stderr_truncated) = join_output(stderr_handle).context("join stderr")?;

    Ok(CommandOutput {
        status,
        stdout,
        stderr,
        stdout_truncated,
        stderr_truncated,
        timed_out,
    })
}

fn join_output(handle: thread::JoinHandle<Result<(Vec<u8>, usize)>>) -> Result<(Vec<u8>, usize)> {
    match handle.join() {
        Ok(result) => result,
        Err(_) => Err(anyhow!("output reader thread panicked")),
    }
}

fn read_stream_limited<R: Read>(mut reader: R, limit: usize) -> Result<(Vec<u8>, usize)> {
    let mut buf = Vec::new();
    let mut truncated = 0usize;
    let mut chunk = [0u8; 8192];

    loop {
        let n = reader.read(&mut chunk).context("read output")?;
        if n == 0 {
            break;
        }
        let remaining = limit.saturating_sub(buf.len());
        if remaining > 0 {
            let keep = n.min(remaining);
            buf.extend_from_slice(&chunk[..keep]);
            truncated += n.saturating_sub(keep);
        } else {
            truncated += n;
        }
    }

    Ok((buf, truncated))
}
