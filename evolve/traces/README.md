# Trace suites

This directory holds the evaluation substrate the fitness plugins consume.

```
traces/
  dev/        # 5 traces visible to the AI; default suite for fitness plugins
  holdout/    # 3 traces hidden from the AI; run only on Pareto-front variants
  smoke.trace # framework-owned G5 trace (lives at this level)
```

Each `*.trace` file is plain text: lines beginning with `#` are headers/comments,
blank lines are ignored, and every other line is a single decimal `u64` key. The
replayer is `src/bin/evolve_hit_rate.rs` (invoked by `scripts/fitness/hit-rate.py`).

## Held-out hidden flag

Holdout traces are hidden from the AI's context bundle. Their frontmatter
declares `hidden: true`. The context-bundle assembler (issue #13) is documented
not to read `traces/holdout/` or any `*.profile.md` carrying `hidden: true`.

Until the assembler exists, the boundary is enforced by convention plus the
directory split: subagent prompts and skills must not read from
`evolve/traces/holdout/`.

## Trace-profile sign-off workflow

Every trace ships with a `*.profile.md` describing its workload shape (skew,
working-set size, scan behavior, temporal locality). Authorship is hybrid:

1. **AI drafts.** On first encountering a trace, the framework proposes a
   `*.profile.md` next to it. The profile is populated from the trace's
   generator parameters and a qualitative summary.
2. **Maintainer reviews and signs off.** The human reviewer reads the profile,
   adjusts wording or characterization if needed, and adds two frontmatter
   fields:

   ```yaml
   signed_off_by: <github-handle>
   signed_off_at: <YYYY-MM-DD>
   ```

   The sign-off may also live in the commit message that lands the edit.

3. **Profile is immutable for the run.** Once signed off, the profile is not
   rewritten by the framework. If the trace itself changes (e.g. a regenerate),
   the profile must be re-reviewed and re-signed.

The frontmatter `signed_off_by` field is the source of truth — an empty value
means the profile is still a draft.

## Regenerating traces

`scripts/gen-traces.py` is deterministic: re-running produces byte-identical
files. Seeds and parameters live in the script. Profiles are not regenerated.

```bash
python3 evolve/scripts/gen-traces.py             # both suites
python3 evolve/scripts/gen-traces.py --suite dev # only one
```
