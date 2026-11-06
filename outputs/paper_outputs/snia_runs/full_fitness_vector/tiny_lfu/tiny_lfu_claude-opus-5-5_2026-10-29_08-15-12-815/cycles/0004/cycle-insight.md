# Cycle 0004 Insight

## Cohort outcome

Each variant tested one item left open by cycle 0003. v001 put a branch-free conservative update on the window/SLRU aging line. v002 ported the blocked aging sketch onto the u32-link layout. v003 fixed the admission window at 5%. All three are Winners on hit-rate. v002 also won on sketch metadata. None improved per-op-time, and only v002 held it roughly flat.

## Patterns

- The two hit-rate levers cover opposite cache sizes. Conservative update (v001) gains only at the large size, where close mid-range frequencies make small overestimates decide the contest. A larger window (v003) gains only at the small size, where a 1% window cannot hold a recency burst, and it slips slightly at the large size. These look complementary, not overlapping.
- The sketch increment is still the latency bottleneck, but the cause has moved. Going branch-free did not help, because the serial dependency chain through `min` costs more than the well-predicted branches it removed. Shortening that critical path is the lever. Removing branches alone is not.
- The u32-link latency regression held up in a same-run test against the same sketch, so it belongs to the residency layer (link codegen), not to scattered sketch access. This is now an established result, not suspected noise.
- Per-op-time readings are becoming unreliable. v003 slowed down on every op with a byte-identical hot path. Read with cycle 0003's unexplained regressions, this suggests the benchmark's noise floor is close to the size of the effects being measured.
- Merges keep adding up: the aging sketch carried over to the 24-byte node with no interference. Its hit-rate was close to v001's figure but not exactly equal, which is still unexplained.

## Open questions

- Why is v002's hit-rate not identical to v001's when it uses the same copied sketch? Is the copy faulty, or does residency order differ?
- Can a SWAR or pairwise min (or an approximate min) keep conservative update's accuracy at the parent's increment latency?
- Is the u32-link slowdown caused by bounds checks or mask/widen sequences in `unlink`/`push_back`/`on_hit`?
- How much of the recent latency movement is noise? Repeated runs or interleaved A/B would tell.

## Continuation

Merge v001's conservative update with v003's window, but size the window as an absolute floor (for example, max of 1% and roughly 800 entries) so the large size keeps its main-region slots. Then shorten the increment's `min` dependency chain. Also fix codegen in the u32-link paths separately, and check that v002's sketch matches v001's hit-rate exactly. Before trusting any per-op-time verdict, firm up the benchmark with repeated or interleaved runs. Do not pursue branch removal alone as a latency fix, and do not raise the window percentage uniformly.
