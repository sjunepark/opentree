package config

import (
	"os"
	"path/filepath"
	"testing"
)

func TestLoadMerged_RespectsBooleanOverrides(t *testing.T) {
	repoRoot := t.TempDir()
	t.Setenv("HOME", t.TempDir())

	if err := os.MkdirAll(filepath.Join(repoRoot, ".ralph"), 0o755); err != nil {
		t.Fatalf("mkdir: %v", err)
	}
	writeFile(t, filepath.Join(repoRoot, ".ralph", "config.yaml"), `
version: 1
git:
  require_clean: false
`)

	cfg, err := LoadMerged(repoRoot)
	if err != nil {
		t.Fatalf("LoadMerged: %v", err)
	}
	if cfg.Git.RequireClean {
		t.Fatalf("expected git.require_clean=false, got true")
	}
}

func TestLoadMerged_PrecedenceEnvOverProjectOverGlobal(t *testing.T) {
	repoRoot := t.TempDir()
	home := t.TempDir()
	t.Setenv("HOME", home)

	globalPath := filepath.Join(home, ".config", "ralph", "config.yaml")
	writeFile(t, globalPath, `
version: 1
loop:
  max_iterations: 10
`)

	if err := os.MkdirAll(filepath.Join(repoRoot, ".ralph"), 0o755); err != nil {
		t.Fatalf("mkdir: %v", err)
	}
	writeFile(t, filepath.Join(repoRoot, ".ralph", "config.yaml"), `
version: 1
loop:
  max_iterations: 20
`)

	t.Setenv("RALPH_MAX_ITERATIONS", "30")

	cfg, err := LoadMerged(repoRoot)
	if err != nil {
		t.Fatalf("LoadMerged: %v", err)
	}
	if cfg.Loop.MaxIterations != 30 {
		t.Fatalf("expected loop.max_iterations=30, got %d", cfg.Loop.MaxIterations)
	}
}

func TestLoadMerged_ExecutorKindEnvClearsCommand(t *testing.T) {
	repoRoot := t.TempDir()
	t.Setenv("HOME", t.TempDir())

	if err := os.MkdirAll(filepath.Join(repoRoot, ".ralph"), 0o755); err != nil {
		t.Fatalf("mkdir: %v", err)
	}
	writeFile(t, filepath.Join(repoRoot, ".ralph", "config.yaml"), `
version: 1
executor:
  kind: codex
  command: ["codex", "exec", "--sandbox", "danger-full-access", "-"]
`)

	t.Setenv("RALPH_EXECUTOR_KIND", "claude")

	cfg, err := LoadMerged(repoRoot)
	if err != nil {
		t.Fatalf("LoadMerged: %v", err)
	}
	if cfg.Executor.Kind != "claude" {
		t.Fatalf("expected executor.kind=claude, got %q", cfg.Executor.Kind)
	}
	if cfg.Executor.Command != nil {
		t.Fatalf("expected executor.command=nil after kind override, got %#v", cfg.Executor.Command)
	}
}

func writeFile(t *testing.T, path string, contents string) {
	t.Helper()

	if err := os.MkdirAll(filepath.Dir(path), 0o755); err != nil {
		t.Fatalf("mkdir %s: %v", filepath.Dir(path), err)
	}
	if err := os.WriteFile(path, []byte(contents), 0o644); err != nil {
		t.Fatalf("write %s: %v", path, err)
	}
}
