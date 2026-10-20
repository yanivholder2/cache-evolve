# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-10-19T12:27:12+00:00

## timeline

### 12:27:29 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/dash_2026-10-19_11-52-26-211/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- **Stash ordering by proven reuse** is the most reliable hit-rate lever seen so far (cycles 1–2). Demotees go to the stash MRU/Back end and fresh puts go to the LRU/Front end. The two gains add up, so the stash works as a scan-resistant probationary queue.\n- **Using idle capacity beats admission policy** (cycle 4). Retaining items that would otherwise be dropped, by placing them in holes elsewhere (a neighbour stash bucket, or free target/probing slots), gave the first hit-rate gains after cycle 2. It adds no metadata.\n- **State-neutral changes compose cleanly.** Lookup-order changes, stash insert positions and bookkeeping refactors reproduced the expected cache state exactly when combined.\n- **Metadata-size has not moved in any variant so far.** Every lever tried derives placement from the hash or from bucket fullness, so it adds no extra state.\n- **Latency gains come from doing fewer promotion moves or cheaper bucket scans** (cycles 1, 3). Skipping the remove → promote → demote → stash-insert chain reliably shortens the get tail.\n\n## Recurring failures\n\n- **Admission filters on either promotion edge.** MRU placement on stash→target (cycle 2), a second-hit gate (cycle 3) and a native-ownership gate on target→probing (cycle 3) all lost hit-rate. With tiny buckets, items can't stay resident long enough for an extra wait to pay off. Free signals such as slot position or ownership don't track recency well enough.\n- **Placement tweaks that only fire in rare corner cases**, such as balanced placement on stash hits (cycle 1). They are safe but have no measurable effect.\n- **Pure bookkeeping trims** (less redundant hashing or re-scanning) don't move get p50 (cycle 4). The common non-promoting lookup path sets the median.\n- **Fixed lookup-order changes carried across lineages.** Target-first won in cycle 1 but lost after fresh-at-stash-Front placement made stash hits common (cycle 3). The best order depends on the placement policy.\n- **Per-op-time claims without a code-level cause.** Several cycles show latency shifts on code paths the diff never touched, including large shifts across the whole cohort. Treat these as environment noise.\n\n## Open questions\n\n- Do two-choice stash and victim rescue stack? Both touch stash insert. Their hit-rate gains may overlap while their latency costs add up.\n- Can the second stash scan on misses be filtered or short-circuited without losing two-choice retention? Should target/probing-first order be re-tested under two-choice?\n- Can the promotion chain be made cheaper without changing which items get promoted, e.g. in-place swaps or a cap on how far it goes?\n- Would per-slot fingerprints cut scan cost enough to justify the metadata-size cost?\n- Would an independent second hash for the alternate stash bucket retain more than a fixed neighbour stride?\n- What do stash capacity and associativity do? Larger stash buckets might finally let filters work.\n- How do hits split between the stash and the normal tiers? This is still unmeasured, yet it drives both lookup-order and placement outcomes.\n\n## Per-policy notes\n\n- **Stash tier:** this is where most hit-rate leverage sits. Its insert-position policy (fresh at Front, demotees at Back) is now the settled base. Adding a second stash choice raises retention but adds a scan to every miss.\n- **Stash→target promotion:** keep plain LRU-style promotion that places at the Front. Hotter placement and stricter gating both hurt. The stash-hit promotion path is now the slowest common hit path, and its cost grows as hit-rate rises.\n- **Target→probing promotion:** gating it by ownership lost reuse. Leave admission as is and focus on what each move costs.\n- **Normal (target/probing) buckets:** they have spare holes after promotions, and rescuing stash victims into them helps hit-rate. Fuller buckets make every lookup scan more.\n- **General trade-off:** from cycle 4 on, gains in retention trade directly against get latency. Judge every hit-rate gain against its per-op-time cost, measured against a baseline taken in the same run.\n"
}
```

### 12:27:36 chat

I wrote `cross-cycle-insights.md` in the cycle 0004 cross-cycle folder. It uses the four required sections in order and is built only from the four ledger entries, with no numbers.

**Recurring wins:**
- Ordering the stash by proven reuse, which worked in cycles 1–2 and the gains added up.
- Making use of idle capacity elsewhere instead of tightening admission, which gave cycle 4's hit-rate gains.
- Changes that don't alter cache state combine cleanly.
- Metadata-size never moved in any cycle.
- Latency only improves when there are fewer promotion moves or cheaper bucket scans.

**Recurring failures:**
- Admission filters on either promotion edge.
- Placement tweaks that only trigger in rare corner cases.
- Trimming bookkeeping, which doesn't reach the median get time.
- Reusing a fixed lookup order across lineages: target-first won in cycle 1 but lost in cycle 3.
- Latency changes with no cause in the code, which should be treated as environment noise.

**Open questions:**
- Do the two cycle-4 levers (two-choice stash and rescuing stash victims) add up?
- Can the extra stash scan on every miss be avoided?
- Can each promotion be made cheaper without changing which items are promoted?
- Are small per-key fingerprints worth their metadata cost?
- Would an independent second hash work better than the neighbouring bucket?
- What do stash size and associativity do?
- How do hits split between the stash and the normal buckets? This is still unmeasured.

**Per-policy notes** cover:
- The stash tier.
- Stash→target and target→probing promotion.
- The normal buckets.
- The general hit-rate versus latency trade-off that has applied since cycle 4.


---

- finished: 2026-10-19T12:27:36+00:00
- duration: 23.77s
- status: ok
- input_tokens: 4
- output_tokens: 2166
- cache_read_tokens: 36872
- cache_write_tokens: 9766
- cost_usd: 0.128838
