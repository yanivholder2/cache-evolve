# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0002/cross-cycle
- allow_edits: True
- started: 2026-10-20T12:24:25+00:00

## timeline

### 12:24:42 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0002/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Capacity levers beat movement-rule levers. Making the small/main split adaptive (ARC-style) is the only change so far that improved hit-rate on workload-a, and it added no per-entry metadata.\n- Structural/data-structure fixes that keep policy behaviour unchanged give clean latency wins. Replacing linear scans with hashed indexing kept hit-rate identical and cut `get`/`put` latency sharply. Stacking it on the adaptive-split parent produced the current best lineage (0001 v003 → 0002 v003).\n- Building on the latest winner, one change per cycle, has produced every gain so far.\n\n## Recurring failures\n\n- Movement-rule tweaks layered on the adaptive split have not paid off. The looser graduation threshold was flat, and ghosting main victims was slightly worse. The likely cause is that extra graduations or ghost hits feed the split adaptation and cancel the intended effect. This is unconfirmed.\n- Latency claims measured on scan-based parents were unreliable. Their latency was dominated by O(capacity) scans, so latency attributed to policy changes in early cycles, gains and regressions alike, should be discounted.\n- Infrastructure flakiness (Maven 429s at build time) wasted evaluation slots in cycle 0001. A rejection caused by a build failure says nothing about the hypothesis.\n- \"Winner\" status within a weak cohort does not by itself show the variant beat its parent. Check its result against its parent directly.\n\n## Open questions\n\n- Where does `small_size` settle under the adaptive split? Is the gain from adaptation itself, or just from moving to a better fixed split? No variant has reported or compared against a fixed split near the settled value.\n- The resize triggers are lopsided: \"grow main on graduation\" fires almost all the time. Would rebalancing them (or making grow-small fire less often) improve hit-rate?\n- Could a main-victim ghost kept separate from the split adaptation, so it does not trigger \"grow main\", recover hit-rate from the ghost-main idea?\n- On the hashed base, do movement-rule changes still cost latency, or was that cost only a side effect of the scans?\n- `metadata-size` only counts `Entry<K,V>` and misses structural overhead (indexes, ghost structures). Real memory cost of the current lineage is under-reported.\n\n## Per-policy notes\n\n- S3-FIFO baseline (fixed 25/75 small/main split): the fixed split is not optimal for workload-a, and capacity adaptation beats it.\n- S3-FIFO + adaptive split: hit-rate leader. It seems sensitive to anything that changes graduation or ghost-hit rates, since those signals drive resizing.\n- S3-FIFO + adaptive split + hashed indexing: current parent and latency baseline. Compare all future latency against it. Its hidden structural memory cost is not captured by the metadata metric.\n- Graduation-threshold variants: dropped after a flat result on the adaptive base.\n- Ghost-main-victim variants: retry only in a form decoupled from the split adaptation.\n"
}
```

### 12:24:51 chat

I wrote `cross-cycle-insights.md` from the two-cycle ledger. It has the four required sections, in order.

- **Recurring wins:** Changing how much room the small and main queues get (the adaptive split) is the only change so far that improved hit-rate. Swapping linear scans for hashed indexing cut latency and left hit-rate unchanged. The best line so far is cycle 0001's adaptive split followed by cycle 0002's hashed refactor.
- **Recurring failures:**
  - Changing the per-key movement rules on top of the adaptive split hasn't helped. Loosening graduation was flat and ghosting main victims was slightly worse, probably because both interfere with the split adaptation.
  - Latency measured on the older scan-based parents can't be trusted.
  - The Maven 429 build failures in cycle 0001 meant two hypotheses were never evaluated.
  - A variant marked "Winner" in a weak cohort can still lose to its parent.
- **Open questions:**
  - Where does `small_size` actually settle, and does the adaptive split beat a fixed split near that value?
  - The two resize triggers are lopsided ("grow main" fires almost all the time).
  - Would a separate ghost for main victims help if it doesn't feed the split adaptation?
  - On the hashed base, do the movement-rule changes still cost latency?
  - The `metadata-size` metric only counts `Entry<K,V>`, so it misses the real memory cost of structural changes.
- **Per-policy notes:** It covers the baseline, the adaptive split, the adaptive split with hashing (the current parent and latency baseline), and the two dropped or deferred movement-rule variants.


---

- finished: 2026-10-20T12:24:52+00:00
- duration: 27.23s
- status: ok
- input_tokens: 4
- output_tokens: 1724
- cache_read_tokens: 34553
- cache_write_tokens: 7050
- cost_usd: 0.097807
