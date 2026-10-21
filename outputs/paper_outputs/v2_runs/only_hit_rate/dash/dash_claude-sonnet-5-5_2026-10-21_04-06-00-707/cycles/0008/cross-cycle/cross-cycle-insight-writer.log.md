# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-10-21T04:38:02+00:00

## timeline

### 04:38:12 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-21_04-06-00-707/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Changing *what gets kept or dropped* beat changing *where things sit*. The two clear movers were the second-sighting admission filter (cycle 3) and reuse-signal victim selection (cycle 6).\n- Using a real reuse signal (hit since insertion) for eviction gave the largest gain since admission. Finer state on top of it (counters, aging, ghost history, tiers) added only small increments.\n- Local, scan-time aging tied to where the evictor looks works. Global timer-driven aging did not.\n- Keeping the admission filter unconditional has held up. It is not redundant with reuse-based eviction.\n\n## Recurring failures\n\n- Placement and position tweaks (insert, demotion, eligibility, two-tier insert, power-of-two buckets, wider probes) were flat or slightly negative from cycle 2 on. Once admission screens one-hit-wonders, ranking within a bucket adds nothing.\n- Refining or stacking admission machinery (counters, doorkeepers, victim exemption, bigger filters, count-min gates, free-slot bypass) lost or tied. The incumbent filter looks like a local optimum, and overlapping admission mechanisms are not independent wins.\n- Scan-order and second-chance tweaks were identical to the parent. History and ghost state gave near-noise gains.\n- Several variants matched their parent's hit-rate exactly, so some overlay edits may have been dead code paths. This weakens negative conclusions about those levers.\n- Winner labels often go to noise-level or even below-parent variants (cycles 4, 5, 8). Selection is relative to the cohort and is not a real-gain signal.\n\n## Open questions\n\n- Do the admission and eviction gains hold on other workloads and cache sizes? All evidence is from workload-a at one size.\n- Is any hit-rate headroom left on workload-a, or is it saturated across admission, placement and eviction state?\n- What do the winners cost in per-op-time and metadata (filter memory, counter aging on scan, ghost table, multi-pass scans)? None of this has been measured. Hit-rate is the only fitness dimension so far.\n- Are there untried levers outside admission, placement and eviction state, such as adaptive sizing or scan-resistance for other access patterns?\n- Are overlay edits reliably on the live path? The authors cannot read the sources.\n\n## Per-policy notes\n\n- Baseline promotion/insertion: MRU placement on hit helped once (cycle 1). Every further placement variant was neutral.\n- Admission (second-sighting one-bit filter): the cycle 3 incumbent. Capacity, aging, counters, doorkeepers and frequency gates did not beat it. It does not stack on parents that already screen inserts.\n- Eviction (reuse-bit victim choice, cycle 6): a large step that is now the base. A two-bit aged counter gave a modest gain (cycle 7). Ghost restoration gave a marginal gain (cycle 8). Tiers gave a modest gain on the one-bit parent. Scan order and global halving gave none or a loss.\n- Current incumbent: the counter-based reuse eviction with the unconditional admission filter. It is a marginal-gain line with unmeasured cost.\n"
}
```

### 04:38:17 chat

I wrote `cross-cycle-insights.md` to the cycle 0008 cross-cycle output directory. It has the four required sections in order, built only from the eight-cycle ledger digest.

- **Recurring wins:**
  - What gets kept or dropped beat where things sit. The second-sighting admission filter (cycle 3) and reuse-based victim selection (cycle 6) were the clear movers.
  - Finer reuse state (counters, aging, ghost history, tiers) added only small gains.
  - Aging tied to the eviction scan helped, while timer-driven aging hurt.
- **Recurring failures:**
  - Placement and position tweaks stayed flat or slightly negative.
  - Further admission machinery lost or tied.
  - Several variants matched their parent's hit-rate exactly, which suggests some edits never ran.
  - Winner labels often went to noise-level or below-parent variants.
- **Open questions:**
  - Whether the gains hold beyond workload-a at one cache size.
  - Whether any hit-rate headroom is left on it.
  - What the winners cost in per-op-time and metadata, which has never been measured.
  - Whether untried levers outside admission, placement and eviction state exist.
  - Whether the overlay edits are reliably on the live code path.
- **Per-policy notes:** these cover the baseline promotion and insertion, the admission filter, and the reuse-based eviction line, plus the current incumbent. That incumbent is counter-based reuse eviction with the unconditional admission filter, a marginal-gain line with unmeasured cost.

I used qualitative terms throughout and no numbers.


---

- finished: 2026-10-21T04:38:17+00:00
- duration: 15.16s
- status: ok
- input_tokens: 4
- output_tokens: 1747
- cache_read_tokens: 39458
- cache_write_tokens: 11204
- cost_usd: 0.066956
