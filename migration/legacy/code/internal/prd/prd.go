package prd

import (
	"encoding/json"
	"errors"
	"fmt"
	"os"
)

type prdFile struct {
	UserStories []struct {
		ID     string `json:"id"`
		Passes bool   `json:"passes"`
	} `json:"userStories"`
}

func AllStoriesPass(path string) (bool, error) {
	b, err := os.ReadFile(path)
	if err != nil {
		return false, err
	}
	var p prdFile
	if err := json.Unmarshal(b, &p); err != nil {
		return false, fmt.Errorf("invalid prd.json: %w", err)
	}
	if p.UserStories == nil {
		return false, errors.New("invalid prd.json: missing userStories")
	}
	for _, s := range p.UserStories {
		if !s.Passes {
			return false, nil
		}
	}
	return true, nil
}
