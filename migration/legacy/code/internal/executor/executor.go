package executor

import (
	"context"
	"errors"
	"fmt"
	"io"
	"os"
	"os/exec"
	"strings"

	"github.com/sjunepark/ralph/internal/config"
)

type Executor struct {
	argv       []string
	isOverride bool
	kind       string
	model      string
}

const claudeSettingsJSON = `{"sandbox":{"enabled":true,"autoAllowBashIfSandboxed":true}}`

func FromConfig(cfg config.ExecutorConfig, model string) (Executor, error) {
	kind := strings.ToLower(strings.TrimSpace(cfg.Kind))
	if kind == "" {
		kind = "codex"
	}

	var argv []string
	isOverride := false
	if len(cfg.Command) > 0 {
		argv = append([]string(nil), cfg.Command...)
		isOverride = true
	} else {
		switch kind {
		case "codex":
			argv = []string{"codex", "exec", "--sandbox", "danger-full-access", "-"}
		case "claude":
			argv = []string{"claude", "-p", "--permission-mode", "acceptEdits"}
		default:
			return Executor{}, fmt.Errorf("unknown executor kind: %q", kind)
		}
	}

	if len(argv) == 0 {
		return Executor{}, errors.New("executor command is empty")
	}

	return Executor{argv: argv, isOverride: isOverride, kind: kind, model: model}, nil
}

func (e Executor) Run(ctx context.Context, repoRoot string, promptText string, output io.Writer) error {
	argv := append([]string(nil), e.argv...)

	var stdin io.Reader = strings.NewReader(promptText)

	if !e.isOverride {
		switch e.kind {
		case "codex":
			if strings.TrimSpace(e.model) != "" {
				argv = insertBeforeDash(argv, "--model", e.model)
			}
		case "claude":
			settingsPath, cleanup, err := writeTempSettingsFile(claudeSettingsJSON)
			if err != nil {
				return err
			}
			defer cleanup()

			argv = append(argv, "--settings", settingsPath)
			if strings.TrimSpace(e.model) != "" {
				argv = append(argv, "--model", e.model)
			}
			argv = append(argv, promptText)
			stdin = strings.NewReader("")
		}
	}

	cmd := exec.CommandContext(ctx, argv[0], argv[1:]...)
	cmd.Dir = repoRoot
	cmd.Stdin = stdin
	cmd.Stdout = output
	cmd.Stderr = output
	if err := cmd.Run(); err != nil {
		return err
	}
	return nil
}

func insertBeforeDash(argv []string, parts ...string) []string {
	idx := -1
	for i, a := range argv {
		if a == "-" {
			idx = i
			break
		}
	}
	if idx == -1 {
		return append(argv, parts...)
	}

	out := make([]string, 0, len(argv)+len(parts))
	out = append(out, argv[:idx]...)
	out = append(out, parts...)
	out = append(out, argv[idx:]...)
	return out
}

func writeTempSettingsFile(contents string) (string, func(), error) {
	f, err := os.CreateTemp("", "ralph-claude-settings-*.json")
	if err != nil {
		return "", nil, err
	}
	path := f.Name()
	if _, err := f.WriteString(contents); err != nil {
		f.Close()
		os.Remove(path)
		return "", nil, err
	}
	if err := f.Close(); err != nil {
		os.Remove(path)
		return "", nil, err
	}

	return path, func() { _ = os.Remove(path) }, nil
}
