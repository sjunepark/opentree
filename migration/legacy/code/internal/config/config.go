package config

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"os"
	"path/filepath"
	"strconv"
	"strings"

	"gopkg.in/yaml.v3"
)

type Config struct {
	Version  int            `yaml:"version"`
	Executor ExecutorConfig `yaml:"executor"`
	Loop     LoopConfig     `yaml:"loop"`
	Git      GitConfig      `yaml:"git"`
	State    StateConfig    `yaml:"state"`
}

type ExecutorConfig struct {
	Kind    string   `yaml:"kind"`
	Command []string `yaml:"command"`
}

type LoopConfig struct {
	MaxIterations int `yaml:"max_iterations"`
}

type GitConfig struct {
	ForbidBranches []string `yaml:"forbid_branches"`
	RequireClean   bool     `yaml:"require_clean"`
	BranchPrefix   string   `yaml:"branch_prefix"`
}

type StateConfig struct {
	LogsDir string `yaml:"logs_dir"`
}

type PartialConfig struct {
	Version  *int                   `yaml:"version"`
	Executor *PartialExecutorConfig `yaml:"executor"`
	Loop     *PartialLoopConfig     `yaml:"loop"`
	Git      *PartialGitConfig      `yaml:"git"`
	State    *PartialStateConfig    `yaml:"state"`
}

type PartialExecutorConfig struct {
	Kind    *string   `yaml:"kind"`
	Command *[]string `yaml:"command"`
}

type PartialLoopConfig struct {
	MaxIterations *int `yaml:"max_iterations"`
}

type PartialGitConfig struct {
	ForbidBranches *[]string `yaml:"forbid_branches"`
	RequireClean   *bool     `yaml:"require_clean"`
	BranchPrefix   *string   `yaml:"branch_prefix"`
}

type PartialStateConfig struct {
	LogsDir *string `yaml:"logs_dir"`
}

func Default() Config {
	return Config{
		Version: 1,
		Executor: ExecutorConfig{
			Kind: "codex",
		},
		Loop: LoopConfig{
			MaxIterations: 25,
		},
		Git: GitConfig{
			ForbidBranches: []string{"main", "master"},
			RequireClean:   true,
			BranchPrefix:   "ralph/",
		},
		State: StateConfig{
			LogsDir: ".ralph/logs",
		},
	}
}

func LoadMerged(repoRoot string) (Config, error) {
	cfg := Default()

	globalPath, err := globalConfigPath()
	if err != nil {
		return Config{}, err
	}
	if fileCfg, ok, err := loadStrictYAML(globalPath); err != nil {
		return Config{}, err
	} else if ok {
		merge(&cfg, fileCfg)
	}

	projectPath := filepath.Join(repoRoot, ".ralph", "config.yaml")
	if fileCfg, ok, err := loadStrictYAML(projectPath); err != nil {
		return Config{}, err
	} else if ok {
		merge(&cfg, fileCfg)
	}

	if err := applyEnvOverrides(&cfg); err != nil {
		return Config{}, err
	}
	cfg.Git.ForbidBranches = withRequiredForbiddenBranches(cfg.Git.ForbidBranches)

	if err := validate(cfg); err != nil {
		return Config{}, err
	}

	return cfg, nil
}

func globalConfigPath() (string, error) {
	home, err := os.UserHomeDir()
	if err != nil {
		return "", err
	}
	return filepath.Join(home, ".config", "ralph", "config.yaml"), nil
}

func loadStrictYAML(path string) (*PartialConfig, bool, error) {
	b, err := os.ReadFile(path)
	if err != nil {
		if errors.Is(err, os.ErrNotExist) {
			return nil, false, nil
		}
		return nil, false, err
	}

	var cfg PartialConfig
	dec := yaml.NewDecoder(bytes.NewReader(b))
	dec.KnownFields(true)
	if err := dec.Decode(&cfg); err != nil {
		if errors.Is(err, io.EOF) {
			return &PartialConfig{}, true, nil
		}
		return nil, false, fmt.Errorf("invalid config (strict) %s: %w", path, err)
	}
	return &cfg, true, nil
}

func merge(dst *Config, src *PartialConfig) {
	if src == nil {
		return
	}

	if src.Version != nil {
		dst.Version = *src.Version
	}

	if src.Executor != nil {
		if src.Executor.Kind != nil {
			if kind := strings.TrimSpace(*src.Executor.Kind); kind != "" {
				dst.Executor.Kind = kind
				if src.Executor.Command == nil {
					dst.Executor.Command = nil
				}
			}
		}
		if src.Executor.Command != nil {
			dst.Executor.Command = cloneStringSlice(*src.Executor.Command)
		}
	}

	if src.Loop != nil && src.Loop.MaxIterations != nil {
		dst.Loop.MaxIterations = *src.Loop.MaxIterations
	}

	if src.Git != nil {
		if src.Git.ForbidBranches != nil {
			dst.Git.ForbidBranches = cloneStringSlice(*src.Git.ForbidBranches)
		}
		if src.Git.RequireClean != nil {
			dst.Git.RequireClean = *src.Git.RequireClean
		}
		if src.Git.BranchPrefix != nil {
			if prefix := strings.TrimSpace(*src.Git.BranchPrefix); prefix != "" {
				dst.Git.BranchPrefix = prefix
			}
		}
	}

	if src.State != nil && src.State.LogsDir != nil {
		if logsDir := strings.TrimSpace(*src.State.LogsDir); logsDir != "" {
			dst.State.LogsDir = logsDir
		}
	}
}

func validate(cfg Config) error {
	if cfg.Version != 1 {
		return fmt.Errorf("unsupported config version: %d", cfg.Version)
	}

	kind := strings.ToLower(strings.TrimSpace(cfg.Executor.Kind))
	switch kind {
	case "codex", "claude":
		// ok
	default:
		return fmt.Errorf("invalid executor.kind: %q (expected codex|claude)", cfg.Executor.Kind)
	}

	if cfg.Executor.Command != nil && len(cfg.Executor.Command) == 0 {
		return errors.New("invalid executor.command: must be a non-empty array when provided")
	}

	if cfg.Loop.MaxIterations <= 0 {
		return fmt.Errorf("invalid loop.max_iterations: %d", cfg.Loop.MaxIterations)
	}

	if strings.TrimSpace(cfg.Git.BranchPrefix) == "" {
		return errors.New("git.branch_prefix must be non-empty")
	}

	if strings.TrimSpace(cfg.State.LogsDir) == "" {
		return errors.New("state.logs_dir must be non-empty")
	}

	return nil
}

func applyEnvOverrides(cfg *Config) error {
	if cfg == nil {
		return nil
	}

	if v, ok := getEnvNonEmpty("RALPH_EXECUTOR_KIND"); ok {
		cfg.Executor.Kind = v
		cfg.Executor.Command = nil
	}

	if raw, ok := getEnvNonEmpty("RALPH_EXECUTOR_COMMAND"); ok {
		var argv []string
		if err := json.Unmarshal([]byte(raw), &argv); err != nil {
			return fmt.Errorf("invalid RALPH_EXECUTOR_COMMAND (expected JSON array of strings): %w", err)
		}
		cfg.Executor.Command = cloneStringSlice(argv)
	}

	if raw, ok := getEnvNonEmpty("RALPH_MAX_ITERATIONS"); ok {
		n, err := strconv.Atoi(raw)
		if err != nil {
			return fmt.Errorf("invalid RALPH_MAX_ITERATIONS %q: %w", raw, err)
		}
		cfg.Loop.MaxIterations = n
	}

	if raw, ok := getEnvNonEmpty("RALPH_GIT_REQUIRE_CLEAN"); ok {
		b, err := strconv.ParseBool(raw)
		if err != nil {
			return fmt.Errorf("invalid RALPH_GIT_REQUIRE_CLEAN %q: %w", raw, err)
		}
		cfg.Git.RequireClean = b
	}

	if raw, ok := getEnvNonEmpty("RALPH_GIT_BRANCH_PREFIX"); ok {
		cfg.Git.BranchPrefix = raw
	}

	if raw, ok := getEnvNonEmpty("RALPH_STATE_LOGS_DIR"); ok {
		cfg.State.LogsDir = raw
	}

	return nil
}

func getEnvNonEmpty(key string) (string, bool) {
	v, ok := os.LookupEnv(key)
	if !ok {
		return "", false
	}
	v = strings.TrimSpace(v)
	if v == "" {
		return "", false
	}
	return v, true
}

func cloneStringSlice(in []string) []string {
	out := make([]string, len(in))
	copy(out, in)
	return out
}

func withRequiredForbiddenBranches(in []string) []string {
	out := make([]string, 0, len(in)+2)
	seen := make(map[string]bool, len(in)+2)
	for _, b := range in {
		b = strings.TrimSpace(b)
		if b == "" {
			continue
		}
		if seen[b] {
			continue
		}
		seen[b] = true
		out = append(out, b)
	}
	for _, b := range []string{"main", "master"} {
		if !seen[b] {
			out = append(out, b)
		}
	}
	return out
}
