# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0006/cross-cycle
- allow_edits: True
- started: 2026-10-19T14:01:35+00:00

## timeline

### 14:01:56 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0006/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights\n\n## Recurring wins\n\n- **Stash as a scan-resistant probationary queue (cycles 1–2).** The stash ranks items by proven reuse: demotees go to the MRU/Back end and fresh puts go to the LRU/Front end. The two gains added up and are still the base of every later parent.\n- **Using idle capacity is the only hit-rate lever that has worked since cycle 2 (cycles 4–6).** The working forms are a two-choice stash (neighbour stash bucket) and rescuing stash-eviction victims into target/probing holes. Their gains stack because they draw on different capacity. Neither adds metadata.\n- **State-neutral changes are reliable and easy to verify.** A key lives in exactly one slot, so changing lookup order or adding lossless filters leaves hit-rate exactly the same (cycles 2, 5, 6). Examples: probing `s1` last, and an exact per-segment borrowed counter that skips the `s1` scan. Use these to recover latency.\n- **Lossless filters stored per segment, not per item (cycle 6)** cut get p50 without growing tracked metadata. Their cost shows up on put, mostly in the tail.\n- **Changes on independent paths combine cleanly.** Stash insert position, lookup order and stash-overflow placement have recombined without interfering.\n- **Metadata-size has never moved** across any variant so far.\n\n## Recurring failures\n\n- **Admission filters on either promotion edge (cycles 2–3).** These include MRU placement on stash→target, a second-hit gate, and an ownership gate on target→probing. Hotter promotion evicts proven residents. Stricter promotion blocks real reuse, because the tiny buckets can't hold items long enough to wait. Current admission looks like a local optimum.\n- **Tweaks to rarely-fired corner cases (cycles 1, 6).** Examples: balanced placement on stash hits, and native-victim-into-probing-hole. Such holes almost never exist in steady state. These changes add hot-path branches for no gain.\n- **Filling holes early (cycle 5).** Putting fresh items straight into target holes fills the normal tier with cold items. Later promotions then pay for eviction cascades that used to be free inserts. Filling holes late (rescue) is cheaper.\n- **Bookkeeping trims on the promotion path (cycle 4)** don't reach get p50. The median is set by the common lookup scan and by the promotion moves themselves.\n- **Using lookup order to fix residency cost (cycles 3, 5, 6).** The best order depends on how hits split across tiers, which changes with placement. The cost of fuller buckets is per scan and doesn't depend on order. Further order changes are exhausted.\n- **Latency claims without a code-level mechanism.** Environment shifts have repeatedly moved get and put on paths that weren't touched (cycle 1 especially). Only credit per-op-time changes that have a causal path in the code.\n\n## Open questions\n\n- Does the borrowed-counter `s1` skip stack on top of two-choice + rescue? Does it still skip most probes once rescue changes stash turnover?\n- Can the counter be maintained without re-hashing, by passing `s0` or a native flag from the caller, so that put p99 comes back?\n- Can in-bucket filtering (tags/fingerprints) shorten the longer scans in rescue-filled buckets? Is the metadata cost worth it?\n- Can the promotion cascade (remove → promote → demote → stash insert) be made cheaper without changing what gets promoted? Options: in-place swaps, or a bounded cascade depth.\n- Structural levers not tried yet: stash capacity/associativity, an independent second hash for `s1` instead of the fixed stride, and eviction order inside target buckets.\n- Does fresh-into-target add anything in a form that always gives way to promoted items?\n- How do hits split between stash and normal tiers under the current placement? This is still unmeasured, and it drives which lookup-order and filter choices are right.\n\n## Per-policy notes\n\n- **Dash-style target/probing/stash (only policy so far).**\n  - Hit-rate depends on stash eviction order and on how much idle capacity is kept in use.\n  - Get latency depends on how many occupied slots each lookup scans. Higher residency trades directly against get p50.\n  - Put cost is now a real axis to track: rescue checks, counter bookkeeping, and cascades into fuller buckets all add to it.\n  - Current lineage: Back-demotion + fresh-at-stash-Front → two-choice stash with `s1` probed last → + rescue → + borrowed-counter gate (to be validated with no re-hashing).\n"
}
```

### 14:02:03 chat

I've written `cross-cycle-insights.md` from the six-cycle ledger, using the four required sections in order.

**Recurring wins**
- Treating the stash as a holding queue for unproven items: demoted items go to its Back end and fresh puts to its Front end.
- Putting idle space to use is the only thing that has raised hit-rate since cycle 2. That means a second stash bucket choice, plus rescuing items evicted from the stash into holes in the target or probing buckets. The two gains add up.
- Changes that leave the cache contents untouched have reliably cut latency: reordering lookups, and an exact per-bucket counter that lets `get` skip the second stash scan.

**Recurring failures**
- Stricter or hotter promotion rules on either promotion step have not helped. Current admission looks like a local optimum.
- Tweaks to code paths that rarely run, and filling holes with fresh items early, either did nothing or slowed later promotions.
- Lookup-order changes have run their course. The latency cost of fuller buckets comes from scanning more slots, whatever the order.
- Latency changes with no cause in the code are treated as environment noise.

**Open questions**
- Whether the counter that skips the second stash scan still works on top of the rescue setup, and whether it can avoid re-hashing keys so put p99 recovers.
- Whether short per-slot tags would speed up scans enough to justify the extra metadata.
- Whether the promotion chain of moves can be made cheaper.
- Stash size and layout, which haven't been tested yet.
- How hits split between the stash and the normal buckets, which is still unmeasured.

**Per-policy notes:** there's only one policy so far, so this section notes what drives hit-rate, get latency and put cost, and traces the current lineage.


---

- finished: 2026-10-19T14:02:03+00:00
- duration: 28.32s
- status: ok
- input_tokens: 4
- output_tokens: 2284
- cache_read_tokens: 38817
- cache_write_tokens: 11842
- cost_usd: 0.148195
