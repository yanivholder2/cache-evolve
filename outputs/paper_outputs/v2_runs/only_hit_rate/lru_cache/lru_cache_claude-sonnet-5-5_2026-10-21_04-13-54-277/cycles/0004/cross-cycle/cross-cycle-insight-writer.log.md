# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-sonnet-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0004/cross-cycle
- allow_edits: True
- started: 2026-10-21T04:28:07+00:00

## timeline

### 04:28:16 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-21_04-13-54-277/cycles/0004/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Scan resistance is the lever that pays off: segmented LRU (v003 in cycle 1) and CLOCK-style second-chance (cycle 2) both gained hit-rate over strict LRU by keeping one-touch keys out of the main set.\n- Growing the protected share of the segmented LRU gained hit-rate in two consecutive cycles, then flattened. Probation is now close to its minimum useful size.\n- Each cycle's Winner came from a small, single-axis change to the current best, not from a combination of ideas.\n\n## Recurring failures\n\n- Weakening the protected segment (a smaller share, or direct ghost-to-protected admission) lost hit-rate.\n- Refinements that sit on rarely-taken paths matched their parent exactly and gave no signal: the probation-MRU ghost list and the protected-eviction second-chance bit.\n- Approximate-recency shortcuts (stamp-based relink skipping) gave up a little hit-rate. Their possible speed benefit was never measured.\n- Strict-LRU micro-optimizations were neutral on hit-rate, so they cannot win while hit-rate is the only measured axis.\n- Single-constant tuning is exhausted. Its returns shrank each cycle.\n\n## Open questions\n\n- Metadata-size and per-op-time have been missing from every fitness vector for four cycles. Is this a harness fault? It blocks judging every compactness or speed claim, and selection runs on hit-rate alone.\n- Every result comes from one scan-heavy trace at one cache size. Do the gains hold on other traces and sizes, especially ones with long-distance reuse or little scan pressure?\n- Would adaptive segment sizing (ARC-style) beat any fixed share?\n- Can segmented LRU combine with CLOCK or frequency ideas? Frequency bits looked incremental over one-bit CLOCK.\n- Does the relink-skip shortcut or CLOCK's cheaper hit path pay off in time? It cannot be known until timing is measured.\n\n## Per-policy notes\n\n- Segmented LRU: the best-supported line so far. A high protected share (about 90–95%) is near its plateau. Ghost-list variants have not helped. Protected-eviction tweaks are not worth more effort. The next gains should come from probation or promotion logic, or from adaptive sizing.\n- CLOCK (one-bit second chance): a small hit-rate gain over strict LRU through a different mechanism. Its claimed size and speed advantages are unverified.\n- Frequency-aware CLOCK: a modest gain, no win. Adds little over one-bit CLOCK on this trace.\n- Strict LRU (baseline): the compact-layout and fast-path ideas stay untested. Do not revisit them until size and time are measured.\n"
}
```

### 04:28:19 chat

I wrote `cross-cycle-insights.md` in the cycle 0004 cross-cycle output directory. It has the four fixed sections in the required order.

The main points it records from the four cycles:

- **Recurring wins:** Scan resistance paid off, through segmented LRU and CLOCK second-chance. A larger protected share also paid off, but the curve is now flat near 90–95%.
- **Recurring failures:** Weakening the protected segment lost hit-rate. Refinements on rarely-used paths, such as the ghost list and the second-chance bit, matched their parent and gave no signal. Single-constant tuning has run out of returns.
- **Open questions:** Metadata-size and per-op-time have been missing for four cycles. All results also come from one scan-heavy trace at one cache size.
- **Per-policy notes:** I grouped these by segmented LRU, CLOCK, frequency-aware CLOCK and strict LRU.

I used only the ledger digest and gave no numbers.


---

- finished: 2026-10-21T04:28:19+00:00
- duration: 12.11s
- status: ok
- input_tokens: 4
- output_tokens: 1385
- cache_read_tokens: 36634
- cache_write_tokens: 8236
- cost_usd: 0.050898
