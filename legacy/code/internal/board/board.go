package board

import (
	"bytes"
	"encoding/json"
	"errors"
	"fmt"
	"io"
	"os"
	"regexp"
	"strings"
)

type Board struct {
	Version int    `json:"version"`
	Cards   []Card `json:"cards"`
}

type Card struct {
	ID          string `json:"id"`
	Title       string `json:"title"`
	Description string `json:"description"`
	Priority    *int   `json:"priority"`
	Status      string `json:"status"`
}

var cardIDPattern = regexp.MustCompile(`^[A-Za-z0-9][A-Za-z0-9_-]*$`)

func LoadStrict(path string) (*Board, error) {
	b, err := os.ReadFile(path)
	if err != nil {
		return nil, err
	}

	dec := json.NewDecoder(bytes.NewReader(b))
	dec.DisallowUnknownFields()

	var out Board
	if err := dec.Decode(&out); err != nil {
		return nil, fmt.Errorf("invalid board.json (strict): %w", err)
	}
	if err := ensureEOF(dec); err != nil {
		return nil, fmt.Errorf("invalid board.json (strict): %w", err)
	}

	if out.Version != 1 {
		return nil, fmt.Errorf("invalid board.json: version must be 1, got %d", out.Version)
	}

	if out.Cards == nil {
		return nil, errors.New("invalid board.json: missing required field: cards")
	}

	ids := make(map[string]bool, len(out.Cards))
	inProgress := 0
	for i, c := range out.Cards {
		id := strings.TrimSpace(c.ID)
		if id == "" {
			return nil, fmt.Errorf("invalid board.json: card[%d] is missing id", i)
		}
		if !cardIDPattern.MatchString(id) {
			return nil, fmt.Errorf("invalid board.json: card %q has invalid id (must match %s)", id, cardIDPattern.String())
		}
		if ids[id] {
			return nil, fmt.Errorf("invalid board.json: duplicate card id %q", id)
		}
		ids[id] = true

		if strings.TrimSpace(c.Title) == "" {
			return nil, fmt.Errorf("invalid board.json: card %q is missing title", id)
		}
		if strings.TrimSpace(c.Description) == "" {
			return nil, fmt.Errorf("invalid board.json: card %q is missing description", id)
		}

		switch c.Status {
		case "todo", "in_progress", "done":
			// ok
		default:
			return nil, fmt.Errorf("invalid board.json: card %q has invalid status %q", id, c.Status)
		}
		if c.Status == "in_progress" {
			inProgress++
		}

		if c.Priority != nil && *c.Priority < 1 {
			return nil, fmt.Errorf("invalid board.json: card %q has invalid priority %d (must be >= 1 or null)", id, *c.Priority)
		}
	}
	if inProgress > 1 {
		return nil, errors.New("invalid board.json: multiple cards are in_progress")
	}

	return &out, nil
}

func (b *Board) AllDone() bool {
	for _, c := range b.Cards {
		if c.Status != "done" {
			return false
		}
	}
	return true
}

func ensureEOF(dec *json.Decoder) error {
	tok, err := dec.Token()
	if err == io.EOF {
		return nil
	}
	if err != nil {
		return err
	}
	return fmt.Errorf("unexpected extra token: %v", tok)
}
