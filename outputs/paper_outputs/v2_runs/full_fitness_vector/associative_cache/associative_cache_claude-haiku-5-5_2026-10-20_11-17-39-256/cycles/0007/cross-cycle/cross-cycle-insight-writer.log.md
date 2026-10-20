# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0007/cross-cycle
- allow_edits: True
- started: 2026-10-20T11:52:59+00:00

## timeline

### 11:53:05 thinking



### 11:53:10 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/associative_cache_2026-10-20_11-17-39-256/cycles/0007/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Retention signals that separate re-referenced keys from one-shot inserts keep helping hit-rate on workload-a. Frequency counters, the second-chance bit, and the 2-bit re-reference value all show this.\n- Behavior-preserving rewrites of an existing mechanism can recover time lost to a state-layout change. The word-level scan over the per-bucket mask recovered put median time that the bit-by-bit walk had lost.\n- Keeping the exact global-minimum victim search has been the stable choice. Every shortcut to it has either lost hit-rate or failed to improve the put tail.\n\n## Recurring failures\n\n- Shortcuts to the victim search (rotating window, cached minimum, cached two-candidate pair, unreferenced-only scan, merged halving) either lost hit-rate or did not shorten the put path. The scan length does not appear to be where the put-side cost lives.\n- Put-tail fixes aimed at a single suspected source have not moved the tail. The aging rounds (rewritten as bulk-add) and the zero-counter early exit both failed to change it.\n- Bucket-local position changes (midpoint promotion, midpoint reinsertion) and the bucket-local ghost ring did not raise hit-rate. Lengthening the ghost ring did not help either. The reinsertion idea itself looks weak here.\n- Fingerprint prefilters add a second scan on get and extra write work on put without a hit-rate gain.\n- Extra decay on top of the periodic halving erased frequency protection too fast. The periodic schedule already bounds aging.\n\n## Open questions\n\n- Where does the put tail come from on full-bucket puts? The aging rounds and the scan have both been ruled out as the dominant source.\n- Do the timing winners and losses survive repeated runs? Many get and put movements were single-run and lacked a matching diff.\n- Why did the doorkeeper gain hit-rate in Cycle 0006 and lose it in Cycle 0007? Is the parent or the exact reject rule responsible?\n- Do the hit-rate gains from frequency, second-chance, and the 2-bit re-reference value hold on traces other than workload-a and at other cache sizes?\n- Does the metadata-size probe count per-bucket and per-slot state? It has reported no change in variants that clearly add state.\n- Is the recurring get-side movement caused by struct size or layout?\n\n## Per-policy notes\n\n- Frequency-based victim selection: the first retention gain on workload-a. It needs the exact minimum search, and the periodic halving schedule should be kept as is.\n- Second-chance reference bit: gained hit-rate and lowered put cost in one cycle, but grew per-item metadata and added a get-side write per hit. The mechanism is not yet confirmed from overlay code.\n- 2-bit re-reference value per slot: gained hit-rate, but its aging rounds raised the put tail. Single-workload evidence so far.\n- Per-bucket reference mask: hit-rate parity with its parent and a metadata saving, paid for with get and put time. The word-level scan recovered part of the put time.\n- Doorkeeper admission gate: contradictory results (gain in Cycle 0006, loss in Cycle 0007). Hold until reconciled. If retried, use a looser admit-on-second-sighting rule and keep its bitset out of the inline struct.\n- Fibonacci hash on bucket index: no hit-rate change, so bucket collisions are not the miss driver. Its timing Winner status is probably noise until re-measured.\n- Narrowed or cached victim-search variants (rotating window, cached pair): not productive; drop as put-side fixes.\n- Bucket-local ghost ring and midpoint variants: not productive; do not tune further.\n"
}
```

### 11:53:13 chat

I wrote `cross-cycle-insights.md` to the cross-cycle output directory, using the four fixed sections in order. I did not read the previous file, and I did not use any numbers the digest didn't give.

The main points:

- **Recurring wins:** reference signals that separate re-referenced keys from one-shot inserts have kept helping hit-rate. Keeping the exact minimum victim search has held up, since every shortcut to it either lost hit-rate or didn't improve the put tail.
- **Recurring failures:** victim-search shortcuts, bucket-local position and ghost-ring changes, and put-tail fixes aimed at one suspected source (the aging rounds, the zero-counter exit) have not paid off.
- **Open questions:** the source of the put tail, whether timing movements survive repeated runs, why the doorkeeper gained in Cycle 0006 and lost in Cycle 0007, and whether the retention gains hold beyond workload-a.
- **Per-policy notes:** one short entry per policy, with the doorkeeper marked as unreconciled.


---

- finished: 2026-10-20T11:53:14+00:00
- duration: 14.86s
- status: ok
- input_tokens: 4
- output_tokens: 2748
- cache_read_tokens: 39777
- cache_write_tokens: 12719
- cost_usd: 0.004316
