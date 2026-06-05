---
name: promote-run
description: Curate a finished evolve run and promote it into the committed outputs/paper_outputs/ folder, then commit, push, and open a draft PR. Use when the user wants to keep/save/archive/promote a run for the paper — e.g. "move this run to paper outputs", "archive this run", "save this run for the paper". Strips the multi-GB caffeine logs so the copy is git-friendly.
---

Take this session's evolve run output, copy a curated (git-friendly) subset into
the committed `outputs/paper_outputs/` folder, then commit, push, and open a
draft PR. This encodes conventions you'd otherwise have to reverse-engineer — do
not re-investigate, just follow and verify.

## Context (verify, don't rediscover)

- `outputs/paper_outputs/` is the curated, committed home for runs worth keeping
  (introduced in PR #179). `.gitignore` ignores all of `outputs/*` **except**
  `!outputs/paper_outputs/`. So a raw run dir under `outputs/` is untracked,
  while a copy under `outputs/paper_outputs/` is trackable.
- A full raw run is multi-GB; the curated copy is a few MB. The bloat is
  `eval/caffeine/rust-logs/*.log` (hundreds of MB each) plus the gradle logs.
  These **must** be dropped — they can't go in git.
- `rsync` is **not** installed. Use `tar` with excludes (streams directly, no
  giant intermediate copy).

## Steps

### 1. Identify the run dir

Auto-detect the newest run dir that is **not** already under `paper_outputs/`:

```
RUN_DIR=$(ls -1dt outputs/*_*/ | grep -v paper_outputs | head -1)
BASENAME=$(basename "$RUN_DIR")
```

If more than one plausible candidate exists, confirm with the user before
proceeding. If the run is still in progress, warn that the curated copy will only
contain completed cycles.

### 2. Back it up first

Before any git operation:

```
cp -a "$RUN_DIR" "/tmp/RUN_BACKUP_$BASENAME" && du -sh "/tmp/RUN_BACKUP_$BASENAME"
```

### 3. Ensure the branch has the paper_outputs convention

If `outputs/paper_outputs/` doesn't exist, or `.gitignore` lacks the
`!outputs/paper_outputs/` line, the branch is outdated:

```
git fetch origin main
```

Then bring the branch up to date — fast-forward if it has no unique commits,
otherwise merge. The untracked run dir is timestamped and won't collide, so this
is safe. Do **not** `git stash` (stash skips untracked files anyway, so it
protects nothing here).

### 4. Curate-copy via tar

```
SRC="$RUN_DIR"; DST="outputs/paper_outputs/$BASENAME"
mkdir -p "$DST"
tar -C "$SRC" \
  --exclude='rust-logs' \
  --exclude='gradle.stdout.log' \
  --exclude='gradle.stderr.log' \
  --exclude='./logs' \
  -cf - . | tar -C "$DST" -xf -
```

Also exclude any **incomplete trailing cycle** — a `cycles/NNNN/` dir with no
`cycle-winners.json` (e.g. a cycle the run was killed mid-way). Add
`--exclude='./cycles/NNNN/'` for each such cycle. Keep everything else:
`leaderboard.md`, `cycles-ledger.jsonl`, `cross-cycle-*`, and per-variant
hypotheses / overlays / `eval` CSVs / fitness / insights.

### 5. Verify before committing

- `du -sh "$DST"` is single-digit MB, not GB.
- `find "$DST" \( -name rust-logs -o -name 'gradle.*.log' \)` returns nothing.
- `git check-ignore "$DST"` returns nothing (i.e. it's trackable).
- `outputs/paper_outputs/` still contains the other pre-existing runs (you
  didn't clobber them).

### 6. Gather title metadata

The PR **title must contain the model, the number of cycles, and the number of
variants (cohort size)**. Determine each:

- **Model**: prefer a per-run config snapshot if present
  (`find "$RUN_DIR" -iname '*config*' -o -iname '*manifest*'`). Otherwise read it
  from the run logs:
  `grep -rhoE 'model=[A-Za-z0-9._-]+' "$RUN_DIR" /tmp/evolve_run.log 2>/dev/null | sort -u`
  (the adapter logs `ClaudeSdkAdapter(model=<model>)`). Use the model **actually
  used by this run**, not the repo's current default — they can differ if the
  config changed after the run.
- **Cycles**: the configured cycle count
  (`grep -rhoE 'cycles=[0-9]+' "$RUN_DIR" /tmp/evolve_run.log 2>/dev/null | sort -u`).
  Count completed cycles separately as the number of `cycle-winners.json` files:
  `find "$DST/cycles" -name cycle-winners.json | wc -l`.
- **Cohort size (variants)**:
  `grep -rhoE 'cohort_size=[0-9]+' "$RUN_DIR" /tmp/evolve_run.log 2>/dev/null | sort -u`.

### 7. Commit, push, open a draft PR

`git add "$DST"`, then commit with a message stating subject, cycles completed,
authoring scope, cohort size, and global-best hit-rate vs baseline (read these
from `leaderboard.md` / `cycles-ledger.jsonl`).

Push with `git push -u origin <this-branch>` (retry with exponential backoff on
network errors). Open a **draft** PR if one doesn't already exist; if a PR for
this branch already exists, update its title instead.

**PR title format:**

```
<subject> paper_outputs — <model>, <N> cycles, <M> variants
```

e.g. `dash paper_outputs — claude-sonnet-4-6, 10 cycles, 3 variants`. Use the
configured cycle count for `<N>`; if the run was cut short, state the completed
count (e.g. "9/10 cycles completed") in the PR body, not the title.

Report the global-best result and the PR link when done.
