package prompts

import _ "embed"

//go:embed templates/iteration_run.md
var IterationRun string

//go:embed templates/iteration_board.md
var IterationBoard string

//go:embed templates/prd_generate_unattended.md
var PRDGenerateUnattended string
