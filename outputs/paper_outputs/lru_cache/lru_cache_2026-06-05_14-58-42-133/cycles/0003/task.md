# Task: cycle-insight-writer

Cycle: 0003
Output dir: /home/user/cache-evolve/outputs/lru_cache_2026-06-05_14-58-42-133/cycles/0003

## Winners

- v002

## Variant insights

- /home/user/cache-evolve/outputs/lru_cache_2026-06-05_14-58-42-133/cycles/0003/variants/v002/insight.md

## Hypotheses (this cycle)

- /home/user/cache-evolve/outputs/lru_cache_2026-06-05_14-58-42-133/cycles/0003/hypotheses/H01.md
- /home/user/cache-evolve/outputs/lru_cache_2026-06-05_14-58-42-133/cycles/0003/hypotheses/H02.md
- /home/user/cache-evolve/outputs/lru_cache_2026-06-05_14-58-42-133/cycles/0003/hypotheses/H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants attacked the same target — one-hit-wonder / scan
pollution of a flat LRU — from three classic angles (SLRU segmentation,
2Q with a ghost queue, and TinyLFU frequency-gated admission). Every
variant raised hit-rate on this trace; the segmented SLRU (v001) landed
as the sole Winner.

## Patterns

- The shared winning idea across the whole cohort is separating "seen
  once" from "proven," and protecting proven keys from transient traffic
  — whether enforced by promotion-on-second-hit, FIFO admission, or a
  frequency estimate.
- None of the variants needed to alter the baseline's O(1) arena/slot
  reuse; all gains came from changing *which* resident is evicted, not
  how eviction mechanics work.
- Gating logic, not capacity tuning, was repeatedly identified as the
  decisive lever (v001 credited the promotion rule over the protected
  cap; v003 credited the admission gate over any restructuring).

## Open questions

- Hit-rate is the only measured dimension, so per-access overhead
  (extra lists, ghost map, count-min sketch) is currently invisible —
  is the winner's edge robust if cost is ever scored?
- The three strategies are largely orthogonal; would combining
  admission-frequency signal with segmentation compound the gains or
  just add overhead?

## Continuation

Pursue SLRU-style segmentation as the proven base for the next cycle and
build hypotheses on top of v001. Worth probing: layering a lightweight
frequency/ghost signal onto the SLRU promotion decision (hybrid of v001
and v002/v003), and varying the probationary/protected split. Drop pure
flat-LRU-with-admission-filter framings as standalone parents unless a
new dimension makes their lower overhead matter.


### Cycle 0002

# Cycle 0002 Insight

## Cohort outcome

This cycle tested the prior continuation two ways: layering a ghost
queue onto the proven SLRU base, and exploring a fully adaptive ARC from
baseline. The SLRU+ghost layering regressed hit-rate, while the
self-tuning ARC won — making ARC the sole Winner and the new proven base.

## Patterns

- The promotion gate remains the load-bearing lever. The variant that
  bypassed it — readmitting a returning ghost key straight into the
  protected segment on a single reappearance — over-promoted low-reuse
  keys and displaced hot ones, regressing below its SLRU parent. The
  Winner respects the gate, promoting only on a genuine second reference.
- Self-tuning beat hand-set structure: an adaptive recency/frequency
  target driven by ghost hits outperformed the fixed SLRU split, even
  though it carried more bookkeeping (a second map, ghost arena, four-list
  threading).
- Ghost memory only pays off when it feeds the promotion decision rather
  than overriding it; the same ghost signal helped (ARC) or hurt
  (SLRU+ghost) depending on whether it short-circuited the second-touch
  rule.

## Open questions

- The frequency-sketch admission gate was never actually evaluated this
  cycle (its variant failed to produce a result), so whether a TinyLFU
  estimate adds anything on top of ARC's adaptivity is still untested.
- Overhead stays invisible under hit-rate-only scoring; ARC's extra
  structures would matter if cost is ever measured.

## Continuation

Build the next cycle's hypotheses on the ARC Winner as parent. Drop the
direct-to-protected ghost-readmission framing — it has now been shown to
over-promote. Worth probing: a frequency-sketch admission gate on top of
ARC (the untested idea from this cycle), and varying ARC's ghost-list
bounds or `p` adaptation rate. Avoid reintroducing fixed-split SLRU as a
standalone parent unless a new scored dimension rewards its lower overhead.


## Rejected variants

- v001 — implement
  agent 'variant-implementer' returned status=ok but recorded 0 input AND 0 output tokens (usage=input_tokens=0, output_tokens=0, cache_read_tokens=0, cache_write_tokens=0, cost_usd=0.000966) — almost always a silent backend failure (session limit, quota, dropped connection). tail of /home/user/cache-evolve/outputs/lru_cache_2026-06-05_14-58-42-133/cycles/0003/variants/v001/attempts/01/variant-implementer.log.md:
  # Agent log: variant-implementer
  
  - adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
  - workspace: /home/user/cache-evolve/outputs/lru_cache_2026-06-05_14-58-42-133/cycles/0003/variants/v001/attempts/01
  - allow_edits: True
  - started: 2026-06-05T15:22:18+00:00
  
  ## timeline
  
  ### 15:25:39 chat
  
  API Error: 529 Overloaded. This is a server-side issue, usually temporary — try again in a moment. If it persists, check status.claude.com.
  
- v003 — implement
  agent 'variant-implementer' returned status=ok but recorded 0 input AND 0 output tokens (usage=input_tokens=0, output_tokens=0, cache_read_tokens=0, cache_write_tokens=0, cost_usd=0.000981) — almost always a silent backend failure (session limit, quota, dropped connection). tail of /home/user/cache-evolve/outputs/lru_cache_2026-06-05_14-58-42-133/cycles/0003/variants/v003/attempts/01/variant-implementer.log.md:
  # Agent log: variant-implementer
  
  - adapter: ClaudeSdkAdapter(model=claude-opus-4-8)
  - workspace: /home/user/cache-evolve/outputs/lru_cache_2026-06-05_14-58-42-133/cycles/0003/variants/v003/attempts/01
  - allow_edits: True
  - started: 2026-06-05T15:22:18+00:00
  
  ## timeline
  
  ### 15:25:46 chat
  
  API Error: 529 Overloaded. This is a server-side issue, usually temporary — try again in a moment. If it persists, check status.claude.com.
  

## Write

Exactly one file: `cycle-insight.md` in the output dir.
