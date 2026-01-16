package main

import (
	"context"
	"flag"
	"fmt"
	"os"

	"github.com/sjunepark/ralph/internal/app"
)

func main() {
	os.Exit(realMain(os.Args[1:]))
}

func realMain(args []string) int {
	if len(args) == 0 {
		printUsage(os.Stderr)
		return 2
	}

	switch args[0] {
	case "-h", "--help", "help":
		printUsage(os.Stdout)
		return 0
	case "run":
		fs := flag.NewFlagSet("run", flag.ContinueOnError)
		fs.SetOutput(os.Stderr)

		var maxIterations int
		var executorKind string
		var model string
		var useCurrentBranch bool
		var branchName string
		fs.IntVar(&maxIterations, "max-iterations", 0, "max iterations (override config)")
		fs.StringVar(&executorKind, "executor", "", "executor override: codex|claude")
		fs.StringVar(&model, "model", "", "model override (executor-specific)")
		fs.BoolVar(&useCurrentBranch, "use-current-branch", false, "use current branch (default: create a new ralph/* branch)")
		fs.StringVar(&branchName, "branch", "", "explicit new branch name (default: ralph/run-<timestamp>)")

		if err := fs.Parse(args[1:]); err != nil {
			return 2
		}

		err := app.Run(context.Background(), app.RunOptions{
			MaxIterations:    maxIterations,
			ExecutorKind:     executorKind,
			Model:            model,
			UseCurrentBranch: useCurrentBranch,
			BranchName:       branchName,
		})
		if err != nil {
			fmt.Fprintln(os.Stderr, err)
			return 1
		}
		return 0
	case "board":
		if len(args) < 2 {
			fmt.Fprintln(os.Stderr, "board requires a subcommand: run")
			return 2
		}
		sub := args[1]
		switch sub {
		case "run":
			fs := flag.NewFlagSet("board run", flag.ContinueOnError)
			fs.SetOutput(os.Stderr)

			var maxIterations int
			var executorKind string
			var model string
			var useCurrentBranch bool
			var branchName string
			fs.IntVar(&maxIterations, "max-iterations", 0, "max iterations (override config)")
			fs.StringVar(&executorKind, "executor", "", "executor override: codex|claude")
			fs.StringVar(&model, "model", "", "model override (executor-specific)")
			fs.BoolVar(&useCurrentBranch, "use-current-branch", false, "use current branch (default: create a new ralph/* branch)")
			fs.StringVar(&branchName, "branch", "", "explicit new branch name (default: ralph/run-<timestamp>)")

			if err := fs.Parse(args[2:]); err != nil {
				return 2
			}

			err := app.BoardRun(context.Background(), app.BoardRunOptions{
				MaxIterations:    maxIterations,
				ExecutorKind:     executorKind,
				Model:            model,
				UseCurrentBranch: useCurrentBranch,
				BranchName:       branchName,
			})
			if err != nil {
				fmt.Fprintln(os.Stderr, err)
				return 1
			}
			return 0
		default:
			fmt.Fprintf(os.Stderr, "unknown board subcommand: %s\n", sub)
			return 2
		}
	default:
		fmt.Fprintf(os.Stderr, "unknown command: %s\n", args[0])
		printUsage(os.Stderr)
		return 2
	}
}

func printUsage(w *os.File) {
	fmt.Fprintln(w, "Usage:")
	fmt.Fprintln(w, "  ralph run [--max-iterations N] [--executor codex|claude] [--model <name>] [--use-current-branch] [--branch <name>]")
	fmt.Fprintln(w, "  ralph board run [--max-iterations N] [--executor codex|claude] [--model <name>] [--use-current-branch] [--branch <name>]")
}
