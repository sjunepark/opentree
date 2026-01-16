# Wipe Checklist (Keep Only `migration/`)

Goal: reboot the repository while preserving the pivot context captured under `migration/`.

## Preflight

- [ ] Open `migration/INDEX.md` and verify all links you care about work.
- [ ] Confirm `migration/COPYLIST.md` is fully checked off.
- [ ] Confirm `migration/legacy/` contains:
  - docs + indexes,
  - prompts + indexes,
  - schemas + index,
  - templates + index,
  - code snapshot + notes,
  - root file snapshot + index.

## Option A (recommended): Create a fresh repo from `migration/`

1. Create a new directory for the reboot (outside this repo).
2. Copy `migration/` into it.
3. Initialize a new git repo and commit the snapshot.

Example (adjust paths as needed):

```bash
mkdir -p ../ralph-reboot
cp -R migration ../ralph-reboot/migration
cd ../ralph-reboot
git init
git add migration
git commit -m "chore(migration): seed reboot from migration snapshot"
```

## Option B: Wipe this repo in-place (keep git history)

Use a dedicated branch so this is reversible:

```bash
git checkout -b reboot-from-migration
```

Then remove everything except `.git/` and `migration/`:

```bash
for p in * .*; do
  case \"$p\" in
    .|..|.git|migration) ;; # keep
    *) rm -rf \"$p\" ;;
  esac
done
```

Finally, commit:

```bash
git add -A
git commit -m "chore(reboot): wipe repo (keep migration/ only)"
```

## After the Wipe

- [ ] Decide the new implementation language/runtime (see `migration/HUMAN_QUESTIONS.md`).
- [ ] Move/rename seed docs if desired (e.g., keep `migration/VISION.md` as-is or promote to root).
- [ ] Create the initial strict task-tree schema and runner skeleton per `migration/GOAL.md`.
- [ ] Add a deterministic guard entrypoint (recommended: `just ci`).
