//! CLI tests for `runner select` command.
//!
//! Spawns the runner binary and verifies exit codes match expected values
//! for complete, open, and stuck tree states.

use std::process::Command;

use runner::exit_codes;
use runner::io::init::{InitOptions, RunnerPaths, init_runner};
use runner::io::tree_store::write_tree;
use runner::tree::default_tree;

#[test]
fn select_complete_exits_with_complete_code() {
    let temp = tempfile::tempdir().expect("tempdir");
    init_runner(temp.path(), &InitOptions { force: false }).expect("init");

    let mut tree = default_tree();
    tree.passes = true;
    let paths = RunnerPaths::new(temp.path());
    write_tree(&paths.tree_path, &tree).expect("write tree");

    let status = Command::new(env!("CARGO_BIN_EXE_runner"))
        .current_dir(temp.path())
        .arg("select")
        .status()
        .expect("runner select");

    assert_eq!(status.code(), Some(exit_codes::COMPLETE));
}
