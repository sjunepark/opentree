//! Shared time budget helpers for deterministic orchestration.

use std::time::{Duration, Instant};

use anyhow::{Result, anyhow};

/// Return the remaining time budget until the provided deadline.
pub fn remaining_budget(deadline: Instant) -> Result<Duration> {
    let remaining = deadline
        .checked_duration_since(Instant::now())
        .unwrap_or(Duration::from_secs(0));
    if remaining.is_zero() {
        return Err(anyhow!("iteration timed out"));
    }
    Ok(remaining)
}
