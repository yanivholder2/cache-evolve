## Recurring wins

- **32-byte aligned struct is a non-negotiable base.** Confirmed in Cycle 2 (halved p99 by eliminating cache-line straddling), reconfirmed as the culprit whenever dropped (Cycles 4, 7). Must be explicitly inherited by every variant; it cannot be assumed.
- **3-swap unconditional promotion loop.** Resolved the single-swap convergence lag (Cycle 4), confirmed optimal ceiling in Cycle 6 (five swaps regressed). Early-break keeps the Zipfian hot-key path at one iteration, so the loop is essentially free for hot items while genuinely helping cold-to-warm convergence.
- **Write-free get paths outperform frequency-annotated paths.** Across Cycles 2–5, every variant that wrote per-get counters introduced tail-latency risk; the write itself (not conditional logic) is the cache-line liability. Purely positional, write-free approaches won in Cycles 5 and 6.
- **Second-chance eviction (write-once flag + 8-slot LRU-among-cold scan).** Introduced Cycle 6, scan direction refined in Cycle 8. Write-once flag captures coarse frequency (ever-touched vs. never-touched) without per-get overhead; LRU-among-cold direction evicts the oldest cold candidate instead of the youngest, preserving recently-inserted items long enough to convert — a free hit-rate gain confirmed in Cycle 8. 8-slot depth is confirmed optimal.
- **Second-chance scan and 3-swap promotion are a confirmed unit.** Cycle 7 exhaustively falsified the decoupling hypothesis: the scan only discriminates warm from cold if promotion keeps hot items away from the scan window. Both mechanisms must travel together.

## Recurring failures

- **Per-get frequency counter writes.** Tried in Cycles 1, 2, 4, 5 across multiple encodings (LFU bubble, neighbour-check guard, lfu_counter overlay). Consistently surfaced p99 pressure. Closed as a hot-path mechanism; only write-once-at-first-touch is viable.
- **Two-tail / frequency-at-eviction-boundary.** Tested in Cycles 4 and 5; overrode clean positional recency, ejecting recently-promoted items before genuine LRU victims. Regressed hit rate both times. Closed.
- **Bucket-count manipulation.** Wider buckets (Cycle 1) raised linear-scan cost; halving bucket count (Cycle 6) raised collision density faster than 3-swap depth could absorb. Both directions regressed and are closed.
- **Two-zone hot/cold partitioning.** Boundary shifted on every insertion, pushing borderline items to the cold zone; hot-zone bypass saved no real work. Definitively closed in Cycle 3 and not revisited.
- **Swap-to-back promotion.** Failed on both hit rate and latency at small associativity in Cycle 2. Closed early and not revisited.
- **Promotion budget beyond three swaps.** Five swaps regressed hit rate in Cycle 6 by displacing items that cycle through the top positions. The useful exploration range for promotion budget is exhausted.

## Open questions

- **Root cause of the Cycle 6 v003 put p99 regression remains undiagnosed.** Cycle 7 ruled out 3-swap write pressure (elimination brought no improvement). Candidates are a scan-fallback artifact, a metadata-layout detail in that variant, or a scan-window size interaction. Should be isolated with a controlled put-heavy workload comparison against the current confirmed base.
- **Can LRU-among-cold eviction be further improved with a frequency-weighted tie-break?** Within the 8-slot cold window, evicting the least-accessed cold item rather than the oldest might lift hit rate without overhead — but this requires a counter, raising the write-on-first-touch question.
- **Flag expiry / stale warm markers.** The write-once "ever-accessed" bit never resets, meaning long-lived but rarely-touched items can permanently shield themselves. Expiring or resetting the flag after N eviction passes might improve cold-item turnover without significant overhead.
- **Measurement floor limits per-op-time resolution.** The 100 ns timer floor has masked latency differentiation throughout; whether any promotion-path improvements are real cannot be assessed without higher operation counts or a finer-grained benchmark.

## Per-policy notes

- **Strict LRU (positional baseline):** Competitive baseline. Gains come from improving eviction-candidate quality (second-chance scan), not from restructuring the recency order itself. 3-swap promotion is the right complement — it accelerates hot-item convergence without touching the LRU eviction decision.
- **LFU / frequency-ordered:** Hit-rate advantage on scan-heavy and LIRS traces is reproducible (Cycles 1–3) but consistently blocked: O(n) scan in Cycle 1, get-path write overhead in Cycles 2–5. All direct LFU variants have been superseded by the second-chance flag, which captures coarse frequency (binary) without per-get writes. Full LFU ordering is not worth revisiting unless a write-free encoding is proposed.
- **Second-chance (current confirmed base):** 3-swap promotion + write-once flag + 8-slot scan + LRU-among-cold direction + 32-byte aligned struct. Mechanically coupled — removing any component degrades either hit rate or latency. The flag placement (first-touch vs. position-0 arrival) is neutral on this workload (Cycle 7). The previous LIFO-among-cold default was discarding too-young items; LRU-among-cold (Cycle 8) fixes this at zero cost. Next tunable frontiers are flag expiry and frequency tie-breaks within the cold window.
- **Two-zone partitioning:** Closed. Boundary instability makes it strictly worse than LRU at these bucket sizes.
