package git

import (
	"bytes"
	"context"
	"fmt"
	"os/exec"
	"strings"
)

func ShowTopLevel(ctx context.Context, repoRoot string) (string, error) {
	out, err := run(ctx, repoRoot, "rev-parse", "--show-toplevel")
	if err != nil {
		return "", err
	}
	return strings.TrimSpace(out), nil
}

func CurrentBranch(ctx context.Context, repoRoot string) (string, error) {
	out, err := run(ctx, repoRoot, "rev-parse", "--abbrev-ref", "HEAD")
	if err != nil {
		return "", err
	}
	return strings.TrimSpace(out), nil
}

func HeadSHA(ctx context.Context, repoRoot string) (string, error) {
	out, err := run(ctx, repoRoot, "rev-parse", "HEAD")
	if err != nil {
		return "", err
	}
	return strings.TrimSpace(out), nil
}

func IsClean(ctx context.Context, repoRoot string) (bool, error) {
	out, err := run(ctx, repoRoot, "status", "--porcelain")
	if err != nil {
		return false, err
	}
	return strings.TrimSpace(out) == "", nil
}

func CheckoutNewBranch(ctx context.Context, repoRoot string, name string) error {
	_, err := run(ctx, repoRoot, "checkout", "-b", name)
	return err
}

func run(ctx context.Context, repoRoot string, args ...string) (string, error) {
	cmd := exec.CommandContext(ctx, "git", args...)
	cmd.Dir = repoRoot
	var stdout bytes.Buffer
	var stderr bytes.Buffer
	cmd.Stdout = &stdout
	cmd.Stderr = &stderr
	if err := cmd.Run(); err != nil {
		return "", fmt.Errorf("git %s failed: %w\n%s", strings.Join(args, " "), err, strings.TrimSpace(stderr.String()))
	}
	return stdout.String(), nil
}
