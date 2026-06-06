# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-06-06_02-20-26-792/cycles/0010/cross-cycle
- allow_edits: True
- started: 2026-06-06T03:23:19+00:00

## timeline

### 03:23:23 thinking



### 03:23:42 tool: Write

```json
{
  "file_path": "cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Partition sizing was the first and largest lever: progressively enlarging\n  main toward a very thin admission filter (~4/96) paid repeatedly until it\n  flattened into a clear knee, after which it was frozen.\n- The proven recency-reordering discipline (move-to-front) pays small but real\n  gains wherever it is newly applied — first on main's resident queue, later on\n  the thin small admission queue under the adaptive split.\n- Ghosting the *stronger* main-evicted population (keys that earned promotion at\n  least once) recovered a dropped reuse signal — the first new win after sizing\n  converged. Prioritizing retention of those strong records over weak one-hit\n  history under a fixed budget added a further marginal gain.\n- Making the partition split *adaptive* — steering the eviction threshold from\n  the ghost `from_main` signal, then adding momentum and a wider clamp — was the\n  first mechanism in many cycles to beat static sizing instead of collapsing\n  back into it.\n\n## Recurring failures\n\n- Protecting, memorizing, or warm-starting weak-signal keys consistently\n  regresses: looser promotion gates, frequency warm-start on re-admission\n  (failed three times), and crowding hotter residents all cost more than they\n  saved.\n- Ghost *depth* never paid — falsified over both the weak small-evicted and the\n  strong main-evicted populations. Re-accesses land inside the original\n  `main_size` window or not at all; the win was always ghost *source quality*,\n  never window length.\n- Gating the direct-to-main ghost re-admission path (small-queue routing,\n  throttling, freshness/recency splits) failed on every axis tried — a ghost hit\n  is itself confirmed reuse, and gating discards that signal.\n- Distinct-on-paper retention knobs collapse into the partition ratio once main\n  is large: frequency-ceiling depth, promotion-threshold strictness, and\n  promotion *timing* (eager vs lazy) all became inert.\n- Disturbing the parent's `evict_main` rotation (in-place decrement, windowed\n  victim selection, tail insertion of re-admitted/promoted keys) regressed — the\n  rotation is load-bearing, refreshing survivor recency.\n- Removing weak `from_main: false` ghosts entirely was a severe regression: the\n  weak population is the *dominant* re-admission source and must keep being\n  ghosted.\n- Global time-based frequency aging over-decays hot keys; the eviction-time\n  rotation already does targeted decay.\n\n## Open questions\n\n- Is the configuration now effectively converged for this workload? Recent\n  cycles yield only sliver gains or full regressions across every probed axis.\n- Does the adaptive feedback-controller framing extend to any other runtime knob\n  (it failed for ghost capacity), or is the partition controller's modest,\n  monotone drift the cap on adaptivity headroom?\n- Is there any way to relax raw ghost *capacity* — the binding constraint — without\n  leaking one-hit-wonders, given depth is dead and weak records must be kept?\n- Is there a mechanism not yet touched at all, distinct from sizing, ghost\n  capacity/depth/source, ordering, victim selection, insertion position, and\n  promotion timing?\n\n## Per-policy notes\n\n- All cycles operate on one S3-FIFO-style policy (thin small admission queue,\n  large main resident queue, ghost history queue, FIFO with a `freq`\n  second-chance counter).\n- Main queue: contention-sensitive — protecting it beats feeding it. Its\n  rotation-on-eviction and front-insertion of re-admitted/promoted keys are both\n  load-bearing. Move-to-front recency adds a sliver over plain FIFO.\n- Small queue: too thin (~4%) and fast-churning for intra-ordering to matter\n  much; the strict `freq > 1` promotion gate sets the filter ceiling, not\n  insertion order. Move-to-front nudges positive only under the adaptive split.\n- Ghost queue: source *quality* and source *retention priority* are live levers;\n  depth, gating, warm-start, and weak-record removal are all dead or harmful.\n  Capacity should stay pinned near `main_size`.\n- Partition split: frozen static optimum ~4/96, then improved by source-steered\n  adaptive thresholding with momentum and a wider clamp.\n"
}
```

### 03:23:47 chat

Wrote `cross-cycle-insights.md` as a fresh projection of the 10-cycle ledger, using the four fixed sections in order.

Key synthesis across cycles:
- **Recurring wins**: partition sizing (until its ~4/96 knee), recency move-to-front wherever newly applied, ghosting the stronger main-evicted population, and source-steered adaptive split with momentum.
- **Recurring failures**: protecting/warm-starting weak-signal keys, ghost depth (falsified on both populations), ghost re-admission gating, knobs that collapse into the partition ratio, disturbing the load-bearing `evict_main` rotation, weak-ghost removal, and global aging.
- **Open questions**: whether the config is converged, limits of the adaptivity framing, relaxing the binding ghost-capacity constraint, and any untouched mechanism.
- **Per-policy notes**: grouped by S3-FIFO component (main, small, ghost, split).

Kept it short for downstream hypothesis-writer context, qualitative only (no invented numbers), and drew solely from the ledger digest.


---

- finished: 2026-06-06T03:23:47+00:00
- duration: 27.97s
- status: ok
- input_tokens: 3
- output_tokens: 2039
- cache_read_tokens: 43798
- cache_write_tokens: 13693
- cost_usd: 0.103012
