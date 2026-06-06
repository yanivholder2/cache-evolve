---
name: promote-run
description: Curate a finished evolve run and promote it into the committed outputs/paper_outputs/ folder, then commit, push, and open a draft PR. Use when the user wants to keep/save/archive/promote a run for the paper — e.g. "move this run to paper outputs", "archive this run", "save this run for the paper". Strips the multi-GB caffeine logs so the copy is git-friendly.
---

Move this session's evolve run into the committed `outputs/paper_outputs/`,
dropping the multi-GB caffeine logs, deleting the original raw run dir once the
copy verifies, then commit, push, and open a PR. The conventions below are
non-obvious — follow, don't rediscover.

## Gotchas

- `.gitignore` ignores `outputs/*` except `!outputs/paper_outputs/`: a raw run
  dir is untracked, a copy under `paper_outputs/` is trackable.
- Layout is **per-subject**: `outputs/paper_outputs/<subject>/<run-dir>/`. The raw
  run dir is named `<subject>_<timestamp>`; the curated copy is **renamed** to
  `<subject>_<model>_<timestamp>` (e.g. `dash_claude-opus-4-8_2026-06-05_18-13-17-700`)
  so the model is legible without grepping logs. Subjects and model ids both
  contain underscores/dashes, so derive the pieces from the manifest/config and
  the timestamp suffix — never split on the first underscore.
- A raw run is multi-GB, the curated copy a few MB. The bloat that must not enter
  git: `eval/caffeine/rust-logs/*.log` and the gradle logs.

## Steps

1. **Identify.** The newest run dir not already under `paper_outputs/`. Confirm
   if there are multiple candidates; warn if it's still running (only completed
   cycles are copied). Resolve the **model** now (config snapshot or the
   `ClaudeSdkAdapter(model=…)` log line) — the destination name needs it.

2. **Curate-copy** into `outputs/paper_outputs/<subject>/<subject>_<model>_<timestamp>/`
   (raw name with the model spliced in before the timestamp), excluding the bloat
   above and any incomplete trailing cycle (a `cycles/NNNN/` with no
   `cycle-winners.json`). Keep everything else. The files inside keep their original
   runtime paths — they record where the run actually executed; do not rewrite them.

3. **Verify** the copy: single-digit MB, no rust-logs or gradle logs remain,
   git-trackable (not ignored), and the other subject folders untouched.

4. **Delete the original.** Only after the copy verifies in step 3, remove the
   raw run dir (`outputs/<subject>_<timestamp>/`) so the run isn't left
   duplicated. Never delete before the copy is confirmed good.

5. **Commit + PR.** Commit (subject, cycles completed, cohort size, global-best
   hit-rate vs baseline — from `leaderboard.md` / `cycles-ledger.jsonl`). Push,
   then open a **draft** PR, or update its title if one exists. Report the
   global-best and PR link.

   **Title:** `<subject> paper_outputs — <model>, <N> cycles, <M> variants`
   (e.g. `dash paper_outputs — claude-sonnet-4-6, 10 cycles, 3 variants`).
   - **model** — the one resolved in step 1 (the model the run actually used,
     not the repo default); matches the `<model>` in the run-dir name.
   - **N** — configured cycle count; completed = number of `cycle-winners.json`
     files. If cut short, state "9/10 completed" in the body, not the title.
   - **M** — cohort size (variants).
