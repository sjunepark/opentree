package board

import (
	"os"
	"path/filepath"
	"strings"
	"testing"
)

func TestLoadStrict_ValidEmptyBoard(t *testing.T) {
	path := writeTempFile(t, `{"version":1,"cards":[]}`)
	if _, err := LoadStrict(path); err != nil {
		t.Fatalf("LoadStrict: %v", err)
	}
}

func TestLoadStrict_MissingCardsField(t *testing.T) {
	path := writeTempFile(t, `{"version":1}`)
	_, err := LoadStrict(path)
	if err == nil || !strings.Contains(err.Error(), "missing required field: cards") {
		t.Fatalf("expected missing cards error, got: %v", err)
	}
}

func TestLoadStrict_InvalidID(t *testing.T) {
	path := writeTempFile(t, `{"version":1,"cards":[{"id":"!bad","title":"t","description":"d","priority":1,"status":"todo"}]}`)
	_, err := LoadStrict(path)
	if err == nil || !strings.Contains(err.Error(), "invalid id") {
		t.Fatalf("expected invalid id error, got: %v", err)
	}
}

func TestLoadStrict_MissingTitle(t *testing.T) {
	path := writeTempFile(t, `{"version":1,"cards":[{"id":"C-1","description":"d","priority":1,"status":"todo"}]}`)
	_, err := LoadStrict(path)
	if err == nil || !strings.Contains(err.Error(), "missing title") {
		t.Fatalf("expected missing title error, got: %v", err)
	}
}

func TestLoadStrict_InvalidPriority(t *testing.T) {
	path := writeTempFile(t, `{"version":1,"cards":[{"id":"C-1","title":"t","description":"d","priority":0,"status":"todo"}]}`)
	_, err := LoadStrict(path)
	if err == nil || !strings.Contains(err.Error(), "invalid priority") {
		t.Fatalf("expected invalid priority error, got: %v", err)
	}
}

func TestLoadStrict_DuplicateIDs(t *testing.T) {
	path := writeTempFile(t, `{"version":1,"cards":[{"id":"C-1","title":"t","description":"d","priority":1,"status":"todo"},{"id":"C-1","title":"t2","description":"d2","priority":2,"status":"todo"}]}`)
	_, err := LoadStrict(path)
	if err == nil || !strings.Contains(err.Error(), "duplicate card id") {
		t.Fatalf("expected duplicate id error, got: %v", err)
	}
}

func TestLoadStrict_MultipleInProgress(t *testing.T) {
	path := writeTempFile(t, `{"version":1,"cards":[{"id":"C-1","title":"t","description":"d","priority":1,"status":"in_progress"},{"id":"C-2","title":"t2","description":"d2","priority":2,"status":"in_progress"}]}`)
	_, err := LoadStrict(path)
	if err == nil || !strings.Contains(err.Error(), "multiple cards are in_progress") {
		t.Fatalf("expected multiple in_progress error, got: %v", err)
	}
}

func TestLoadStrict_UnknownFieldRejected(t *testing.T) {
	path := writeTempFile(t, `{"version":1,"cards":[],"extra":true}`)
	_, err := LoadStrict(path)
	if err == nil || !strings.Contains(err.Error(), "unknown field") {
		t.Fatalf("expected unknown field error, got: %v", err)
	}
}

func writeTempFile(t *testing.T, contents string) string {
	t.Helper()

	dir := t.TempDir()
	path := filepath.Join(dir, "board.json")
	if err := os.WriteFile(path, []byte(contents), 0o644); err != nil {
		t.Fatalf("write %s: %v", path, err)
	}
	return path
}
