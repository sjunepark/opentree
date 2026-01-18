//! Helpers for running child processes with timeouts and bounded output.

use std::io::{Read, Write};
use std::process::{Command, ExitStatus, Stdio};
use std::thread;
use std::time::Duration;

use anyhow::{Context, Result, anyhow};
use tracing::{debug, error, instrument, warn};
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
#[instrument(skip_all, fields(timeout_secs = timeout.as_secs(), output_limit_bytes))]
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

    debug!("spawning child process");
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            error!(err = %e, "failed to spawn command");
            return Err(e).context("spawn command");
        }
    };

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
            warn!(
                timeout_secs = timeout.as_secs(),
                "command timed out, killing"
            );
            timed_out = true;
            child.kill().context("kill command")?;
            child.wait().context("wait command after kill")?
        }
    };

    let (stdout, stdout_truncated) = join_output(stdout_handle).context("join stdout")?;
    let (stderr, stderr_truncated) = join_output(stderr_handle).context("join stderr")?;

    if stdout_truncated > 0 || stderr_truncated > 0 {
        warn!(stdout_truncated, stderr_truncated, "output truncated");
    }

    debug!(exit_code = ?status.code(), timed_out, "command finished");
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

/// Run a command with a timeout, capturing stdout/stderr and optionally streaming stdout to a file.
///
/// Similar to `run_command_with_timeout`, but if `stream_path` is `Some`, writes each line of
/// stdout to the file immediately (flushing after each line) for real-time observability.
/// The full stdout is still returned in `CommandOutput`.
#[instrument(skip_all, fields(timeout_secs = timeout.as_secs(), output_limit_bytes, streaming = stream_path.is_some()))]
pub fn run_command_with_stream(
    mut cmd: Command,
    stdin: Option<&[u8]>,
    timeout: Duration,
    output_limit_bytes: usize,
    stream_path: Option<&std::path::Path>,
) -> Result<CommandOutput> {
    if stdin.is_some() {
        cmd.stdin(Stdio::piped());
    } else {
        cmd.stdin(Stdio::null());
    }
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    debug!("spawning child process");
    let mut child = match cmd.spawn() {
        Ok(c) => c,
        Err(e) => {
            error!(err = %e, "failed to spawn command");
            return Err(e).context("spawn command");
        }
    };

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

    // Prepare stream file if requested
    let stream_file = if let Some(path) = stream_path {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("create stream dir {}", parent.display()))?;
        }
        let file = std::fs::File::create(path)
            .with_context(|| format!("create stream file {}", path.display()))?;
        Some(std::sync::Mutex::new(std::io::BufWriter::new(file)))
    } else {
        None
    };

    let stream_file = std::sync::Arc::new(stream_file);
    let stream_file_clone = stream_file.clone();

    // Read stdout with optional streaming
    let stdout_handle = thread::spawn(move || {
        read_stream_limited_with_tee(stdout, output_limit_bytes, stream_file_clone)
    });
    let stderr_handle = thread::spawn(move || read_stream_limited(stderr, output_limit_bytes));

    let mut timed_out = false;
    let status = match child.wait_timeout(timeout).context("wait for command")? {
        Some(status) => status,
        None => {
            warn!(
                timeout_secs = timeout.as_secs(),
                "command timed out, killing"
            );
            timed_out = true;
            child.kill().context("kill command")?;
            child.wait().context("wait command after kill")?
        }
    };

    let (stdout, stdout_truncated) = join_output(stdout_handle).context("join stdout")?;
    let (stderr, stderr_truncated) = join_output(stderr_handle).context("join stderr")?;

    if stdout_truncated > 0 || stderr_truncated > 0 {
        warn!(stdout_truncated, stderr_truncated, "output truncated");
    }

    debug!(exit_code = ?status.code(), timed_out, "command finished");
    Ok(CommandOutput {
        status,
        stdout,
        stderr,
        stdout_truncated,
        stderr_truncated,
        timed_out,
    })
}

/// Read a stream with a size limit, optionally tee-ing to a file.
fn read_stream_limited_with_tee<R: Read>(
    reader: R,
    limit: usize,
    stream_file: std::sync::Arc<Option<std::sync::Mutex<std::io::BufWriter<std::fs::File>>>>,
) -> Result<(Vec<u8>, usize)> {
    use std::io::BufRead;

    let mut buf_reader = std::io::BufReader::new(reader);
    let mut collected = Vec::new();
    let mut truncated = 0usize;

    loop {
        let mut line = Vec::new();
        let n = buf_reader
            .read_until(b'\n', &mut line)
            .context("read line")?;
        if n == 0 {
            break;
        }

        // Tee to stream file if provided
        if let Some(ref mutex) = *stream_file
            && let Ok(mut writer) = mutex.lock()
        {
            // Write and flush immediately for real-time visibility
            if let Err(e) = writer.write_all(&line) {
                warn!(err = %e, "failed to write to stream file");
            } else if let Err(e) = writer.flush() {
                warn!(err = %e, "failed to flush stream file");
            }
        }

        // Collect with limit
        let remaining = limit.saturating_sub(collected.len());
        if remaining > 0 {
            let keep = n.min(remaining);
            collected.extend_from_slice(&line[..keep]);
            truncated += n.saturating_sub(keep);
        } else {
            truncated += n;
        }
    }

    Ok((collected, truncated))
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
