# Wipe Checklist (Archived)

This checklist was used while rebooting the repository around an initial `migration/` seed pack.

The repo has since been unpacked and the `migration/` directory no longer exists. Keep this document as
historical context only.

## Preflight

- [ ] Open `docs/migration/INDEX.md` and verify all links you care about work.
- [ ] Confirm `docs/migration/COPYLIST.md` is fully checked off.
- [ ] Confirm preserved snapshots exist:
  - docs + indexes,
  - prompts + indexes,
  - schemas + index,
  - templates + index,
  - code snapshot + notes,
  - root file snapshot + index.

## Option A (recommended): Create a fresh repo from the unpacked snapshot

1. Create a new directory for the reboot (outside this repo).
2. Copy the repo contents into it.
3. Initialize a new git repo and commit the snapshot.

Example (adjust paths as needed):

```bash
mkdir -p ../ralph-reboot
cp -R . ../ralph-reboot
cd ../ralph-reboot
git init
git add -A
git commit -m "chore(reboot): seed repo from snapshot"
```

## Option B: Wipe this repo in-place (keep git history)

Use a dedicated branch so this is reversible:

```bash
git checkout -b reboot-from-migration
```

Then remove everything except `.git/`:

```bash
for p in * .*; do
  case \"$p\" in
    .|..|.git) ;; # keep
    *) rm -rf \"$p\" ;;
  esac
done
```

Finally, commit:

```bash
git add -A
git commit -m "chore(reboot): wipe repo (keep .git/ only)"
```

## After the Wipe

- [ ] Decide the new implementation language/runtime (see `HUMAN_QUESTIONS.md`).
- [ ] Create the initial strict task-tree schema and runner skeleton per `GOAL.md`.
- [ ] Add a deterministic guard entrypoint (recommended: `just ci`).
