//! Stable exit codes for runner CLI commands.

/// Command succeeded or an open leaf was selected.
pub const OK: i32 = 0;
/// Command failed due to invalid layout/config/tree/run identity or other errors.
pub const INVALID: i32 = 1;
/// `runner select` found no open leaf (tree complete).
pub const COMPLETE: i32 = 2;
/// `runner select` or `runner step` encountered a stuck leaf.
pub const STUCK: i32 = 3;
