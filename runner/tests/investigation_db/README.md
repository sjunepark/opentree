# DB Investigation Tests

This directory is for database-backed investigation tests.

- These tests must be marked `#[ignore]`.
- Wire modules via `runner/tests/investigation_db.rs`.

Run with:

```bash
cargo test -p runner --test investigation_db -- --ignored
```
