package app

import (
	"bufio"
	"context"
	"errors"
	"fmt"
	"io"
	"os"
	"os/exec"
	"path/filepath"
	"strings"
	"time"

	"github.com/sjunepark/ralph/internal/board"
	"github.com/sjunepark/ralph/internal/config"
	"github.com/sjunepark/ralph/internal/executor"
	"github.com/sjunepark/ralph/internal/git"
	"github.com/sjunepark/ralph/internal/prd"
	"github.com/sjunepark/ralph/internal/prompts"
)

type RunOptions struct {
	MaxIterations    int
	ExecutorKind     string
	Model            string
	UseCurrentBranch bool
	BranchName       string
}

type BoardRunOptions struct {
	MaxIterations    int
	ExecutorKind     string
	Model            string
	UseCurrentBranch bool
	BranchName       string
}

func Run(ctx context.Context, opts RunOptions) error {
	repoRoot, err := requireRepoRoot(ctx)
	if err != nil {
		return err
	}

	cfg, err := config.LoadMerged(repoRoot)
	if err != nil {
		return err
	}
	applyOverrides(&cfg, opts.MaxIterations, opts.ExecutorKind)

	if err := preflightAndSelectBranch(ctx, repoRoot, cfg.Git, opts.UseCurrentBranch, opts.BranchName); err != nil {
		return err
	}

	prdPath := filepath.Join(repoRoot, ".ralph", "prd.json")
	if _, err := os.Stat(prdPath); err != nil {
		if errors.Is(err, os.ErrNotExist) {
			return fmt.Errorf("missing required file: %s", prdPath)
		}
		return err
	}

	allPass, err := prd.AllStoriesPass(prdPath)
	if err != nil {
		return err
	}
	if allPass {
		fmt.Fprintln(os.Stderr, "All stories already pass; nothing to do.")
		return nil
	}

	ex, err := executor.FromConfig(cfg.Executor, opts.Model)
	if err != nil {
		return err
	}

	runDir, err := newRunLogDir(repoRoot, cfg.State.LogsDir, "run")
	if err != nil {
		return err
	}

	for i := 1; i <= cfg.Loop.MaxIterations; i++ {
		clean, err := git.IsClean(ctx, repoRoot)
		if err != nil {
			return err
		}
		if !clean {
			return errors.New("working tree must be clean before each iteration")
		}

		beforeHead, err := git.HeadSHA(ctx, repoRoot)
		if err != nil {
			return err
		}

		logPath := filepath.Join(runDir, fmt.Sprintf("iter-%03d.log", i))
		exitCode, err := runOneIteration(ctx, repoRoot, ex, prompts.IterationRun, logPath)
		if err != nil {
			return err
		}

		afterHead, err := git.HeadSHA(ctx, repoRoot)
		if err != nil {
			return err
		}

		allPass, err := prd.AllStoriesPass(prdPath)
		if err != nil {
			return err
		}
		if allPass {
			fmt.Fprintln(os.Stderr, "All stories now pass.")
			return nil
		}

		if exitCode == 0 && afterHead == beforeHead {
			return fmt.Errorf("iteration %d: executor exited 0 but did not create a commit (no progress)", i)
		}
	}

	return fmt.Errorf("max iterations reached (%d)", cfg.Loop.MaxIterations)
}

func BoardRun(ctx context.Context, opts BoardRunOptions) error {
	repoRoot, err := requireRepoRoot(ctx)
	if err != nil {
		return err
	}

	cfg, err := config.LoadMerged(repoRoot)
	if err != nil {
		return err
	}
	applyOverrides(&cfg, opts.MaxIterations, opts.ExecutorKind)

	if err := preflightAndSelectBranch(ctx, repoRoot, cfg.Git, opts.UseCurrentBranch, opts.BranchName); err != nil {
		return err
	}

	boardPath := filepath.Join(repoRoot, ".ralph", "board.json")
	b, err := board.LoadStrict(boardPath)
	if err != nil {
		return err
	}
	if b.AllDone() {
		fmt.Fprintln(os.Stderr, "All cards already done; nothing to do.")
		return nil
	}

	ex, err := executor.FromConfig(cfg.Executor, opts.Model)
	if err != nil {
		return err
	}

	runDir, err := newRunLogDir(repoRoot, cfg.State.LogsDir, "board-run")
	if err != nil {
		return err
	}

	for i := 1; i <= cfg.Loop.MaxIterations; i++ {
		clean, err := git.IsClean(ctx, repoRoot)
		if err != nil {
			return err
		}
		if !clean {
			return errors.New("working tree must be clean before each iteration")
		}

		beforeHead, err := git.HeadSHA(ctx, repoRoot)
		if err != nil {
			return err
		}

		logPath := filepath.Join(runDir, fmt.Sprintf("iter-%03d.log", i))
		exitCode, err := runOneIteration(ctx, repoRoot, ex, prompts.IterationBoard, logPath)
		if err != nil {
			return err
		}

		afterHead, err := git.HeadSHA(ctx, repoRoot)
		if err != nil {
			return err
		}

		b, err := board.LoadStrict(boardPath)
		if err != nil {
			return err
		}
		if b.AllDone() {
			fmt.Fprintln(os.Stderr, "All cards now done.")
			return nil
		}

		if exitCode == 0 && afterHead == beforeHead {
			return fmt.Errorf("iteration %d: executor exited 0 but did not create a commit (no progress)", i)
		}
	}

	return fmt.Errorf("max iterations reached (%d)", cfg.Loop.MaxIterations)
}

func applyOverrides(cfg *config.Config, maxIterations int, executorKind string) {
	if maxIterations > 0 {
		cfg.Loop.MaxIterations = maxIterations
	}
	if strings.TrimSpace(executorKind) != "" {
		cfg.Executor.Kind = strings.TrimSpace(executorKind)
		cfg.Executor.Command = nil
	}
}

func requireRepoRoot(ctx context.Context) (string, error) {
	cwd, err := os.Getwd()
	if err != nil {
		return "", err
	}

	ralphDir := filepath.Join(cwd, ".ralph")
	if st, err := os.Stat(ralphDir); err != nil || !st.IsDir() {
		return "", fmt.Errorf("must run from repo root containing .ralph/: %s", cwd)
	}

	top, err := git.ShowTopLevel(ctx, cwd)
	if err != nil {
		return "", err
	}

	if !samePath(top, cwd) {
		return "", fmt.Errorf("must run from git repo root (%s), got: %s", top, cwd)
	}

	return cwd, nil
}

func samePath(a, b string) bool {
	aAbs, errA := filepath.Abs(a)
	bAbs, errB := filepath.Abs(b)
	if errA == nil {
		a = aAbs
	}
	if errB == nil {
		b = bAbs
	}
	a = filepath.Clean(a)
	b = filepath.Clean(b)
	return a == b
}

func preflightAndSelectBranch(ctx context.Context, repoRoot string, gitCfg config.GitConfig, useCurrentBranch bool, explicitBranchName string) error {
	if gitCfg.RequireClean {
		clean, err := git.IsClean(ctx, repoRoot)
		if err != nil {
			return err
		}
		if !clean {
			return errors.New("working tree must be clean (git status --porcelain is non-empty)")
		}
	}

	branch, err := git.CurrentBranch(ctx, repoRoot)
	if err != nil {
		return err
	}

	forbidden := make(map[string]bool, len(gitCfg.ForbidBranches))
	for _, b := range gitCfg.ForbidBranches {
		forbidden[b] = true
	}

	if forbidden[branch] {
		fmt.Fprintf(os.Stderr, "Refusing to run on %q.\n", branch)
		return checkoutNewBranch(ctx, repoRoot, gitCfg.BranchPrefix, explicitBranchName)
	}

	if useCurrentBranch {
		return nil
	}

	return checkoutNewBranch(ctx, repoRoot, gitCfg.BranchPrefix, explicitBranchName)
}

func checkoutNewBranch(ctx context.Context, repoRoot string, prefix string, explicitBranchName string) error {
	reader := bufio.NewReader(os.Stdin)
	name := strings.TrimSpace(explicitBranchName)
	if name == "" {
		name = prefix + "run-" + time.Now().UTC().Format("20060102-150405")
	}
	if !strings.HasPrefix(name, prefix) {
		return fmt.Errorf("branch name must start with %q, got %q", prefix, name)
	}

	fmt.Fprintf(os.Stderr, "Creating new branch: %s\n", name)
	if err := git.CheckoutNewBranch(ctx, repoRoot, name); err == nil {
		return nil
	} else if strings.TrimSpace(explicitBranchName) != "" || !stdinIsTerminal() {
		return err
	}

	// Interactive fallback (e.g., branch exists).
	for {
		n, perr := promptBranchName(reader, prefix)
		if perr != nil {
			return perr
		}
		fmt.Fprintf(os.Stderr, "Creating new branch: %s\n", n)
		if err := git.CheckoutNewBranch(ctx, repoRoot, n); err == nil {
			return nil
		} else {
			fmt.Fprintln(os.Stderr, err)
		}
	}
}

func stdinIsTerminal() bool {
	st, err := os.Stdin.Stat()
	if err != nil {
		return false
	}
	return (st.Mode() & os.ModeCharDevice) != 0
}

func promptBranchName(r *bufio.Reader, prefix string) (string, error) {
	if strings.TrimSpace(prefix) == "" {
		prefix = "ralph/"
	}
	defaultName := prefix + "run-" + time.Now().UTC().Format("20060102-150405")

	for {
		fmt.Fprintf(os.Stderr, "New branch name (must start with %q) [%s]: ", prefix, defaultName)
		line, err := r.ReadString('\n')
		if err != nil {
			if errors.Is(err, io.EOF) {
				line = ""
			} else {
				return "", err
			}
		}
		name := strings.TrimSpace(line)
		if name == "" {
			name = defaultName
		}
		if !strings.HasPrefix(name, prefix) {
			fmt.Fprintln(os.Stderr, "Branch name must start with prefix.")
			continue
		}
		return name, nil
	}
}

func runOneIteration(ctx context.Context, repoRoot string, ex executor.Executor, promptText string, logPath string) (int, error) {
	if err := os.MkdirAll(filepath.Dir(logPath), 0o755); err != nil {
		return 0, err
	}
	f, err := os.Create(logPath)
	if err != nil {
		return 0, err
	}
	defer f.Close()

	mw := io.MultiWriter(os.Stdout, f)
	fmt.Fprintf(mw, "[ralph] log: %s\n", logPath)

	exitCode := 0
	if err := ex.Run(ctx, repoRoot, promptText, mw); err != nil {
		var exitErr *exec.ExitError
		if errors.As(err, &exitErr) {
			exitCode = exitErr.ExitCode()
			fmt.Fprintf(mw, "[ralph] executor exited non-zero (code %d); continuing\n", exitErr.ExitCode())
		} else {
			return 0, fmt.Errorf("executor failed to run: %w", err)
		}
	}

	clean, err := git.IsClean(ctx, repoRoot)
	if err != nil {
		return exitCode, err
	}
	if !clean {
		return exitCode, errors.New("executor left working tree dirty (expected a clean tree with changes committed)")
	}

	return exitCode, nil
}

func newRunLogDir(repoRoot string, logsDir string, prefix string) (string, error) {
	if strings.TrimSpace(logsDir) == "" {
		logsDir = ".ralph/logs"
	}
	base := filepath.Join(repoRoot, logsDir)
	if err := os.MkdirAll(base, 0o755); err != nil {
		return "", err
	}
	runID := time.Now().UTC().Format("20060102-150405")
	p := filepath.Join(base, fmt.Sprintf("%s-%s", prefix, runID))
	if err := os.MkdirAll(p, 0o755); err != nil {
		return "", err
	}
	return p, nil
}
