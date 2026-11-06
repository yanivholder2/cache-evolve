# Cycle 0002 Insight

## Cohort outcome

Two variants refined last cycle's scan-resistant policies: bimodal insertion on top of LIP (v001) and a 2-bit GCLOCK with a bounded sweep (v002). The third variant (v003) went after the lookup scan with a dense key mirror. v001 is the only Winner, and only just: its hit-rate summary edged up because of one cell (hm_0 at the 128-wide size), while it lost slightly on web_0 and put latency got worse.

## Patterns

- Hit-rate gains from changing the replacement rule are now small, and each one trades between workloads. BIP's blind MRU admission gives back scan resistance on web_0. Extra GCLOCK levels add only a little hit rate, though they did so at every point. The easy scan-resistance gains were taken in cycle 0001.
- Put latency is still set by the mechanism, not by policy dispatch. Removing the per-op string compare (v001) had no visible effect, because the mid-bucket memmove dominates. Capping the sweep (v002) and scanning dense keys (v003) both helped the put tail. Each also made the typical put slower, through more full-window sweeps or writes to two allocations.
- The get p50 summary got worse in both speed-oriented variants. In v003 the cause was clearly locality: a separate key Vec adds a second memory stream to every hit, and on short scans that costs more than density saves. Splitting hot data across allocations is the wrong shape of fix.
- Tail and median now move in opposite directions. Changes that bound worst-case work (sweep cap, dense absence scan) improve p99, and every one so far has added per-op cost at p50.
- Metadata-size stayed flat again. v003 showed that this dimension cannot see memory held in side arrays inside the bucket.

## Open questions

- Is v002's small get p50 regression real or noise? Its get-path change was tiny.
- Can a period that scales with bucket size, or a reuse-aware MRU admission signal, keep BIP's small-bucket gain without losing web_0?
- Would a per-slot fingerprint or tag array stored in the same cache line, or a dense reference-bit array for the sweep, cut get p50 where a separate key Vec did not?

## Continuation

Keep v002's 16-slot sweep cap and clear-on-insert as the base mechanism for the put tail. Treat further replacement-rule tweaks as low yield unless they target a specific workload (for example, BIP admission that scales with bucket size). Spend most of the next cycle on get p50 with layouts that keep the key and the hit-time data together (inline tags or fingerprints, SIMD-friendly blocks), and drop the separate-Vec key mirror. For LIP-style insertion, look for a cheaper insert-at-depth rather than trimming dispatch overhead.
