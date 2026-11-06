# Cycle 0009 Insight

## Cohort outcome

Each variant tried to remove a whole unit of work. v001 (from 0007/v003) cached a per-bucket "confirmed absent" result so a put could skip the tag scan. v002 (from 0008/v002) skipped the MRU move on hits in the top eighth of the bucket (lazy promotion). v003 (from 0008/v002) replaced the cache-level SipHash bucket index with one multiplicative hash that supplies both the bucket bits and the tag. All three are Winners. v002 and v003 are the first clear speed wins since cycle 0006: get and put both got faster at p50 and p99, and hit rate stayed well inside budget. v001 counts as a Winner only because its invariants held. Put got clearly slower and get p99 rose.

## Patterns

- Cycle 0008's question about where the miss-path time goes now has an answer: a large share sat above the bucket, in a redundant SipHash per op. Cycles 5–8 tuned in-bucket loops and never reached it. v003's gain is the most credible of the series because it shows up on both ops and both percentiles, and both ops shared the hash it removed.
- "Remove whole work" only wins when the removed work costs more than the state needed to skip it. v002 needed no state and won. v001 added fields, compares and stores to skip a cheap scan that was already in L1, and lost. The tag scan is now cheap enough that it should not be targeted again.
- Lazy promotion worked where earlier hit-path rewrites failed (`rotate_left`, `ptr::copy`). It removes the move for near-MRU hits instead of re-implementing the move. Reordering inside the top window is far from the eviction end, so hit rate barely moved.
- Writes on read paths are costly. v001 made `get_position` store into the bucket, which dirtied the cache line on every miss, and it grew the stride from 80 to 88 bytes. The get-tail rise is consistent with that. Keep lookups read-only and keep the stride pinned.
- Re-mapping buckets with a multiplicative hash on bits 24–55 did not cluster strided block-address keys. Hit rate moved slightly upward. A cheap hash is safe for these traces.
- v002's put gain is partly drift: the put-miss path is unchanged and only put-update benefits. Same-session repeat runs are still required.

## Open questions

- Do v002 and v003 stack when merged? Their effects should be independent (the hash cost above the bucket, the hit-path memmoves inside it).
- Does widening the lazy-promotion window to `n/4` skip more moves without reaching the quarter-depth insertion region, where eviction order starts to change?
- Now that both hashing and hit-path moves are reduced, what is the largest cost left: the full-key compare, the fused rotation on put-miss, or bucket indexing (`%` vs mask)?
- Does stride by itself cost anything? This needs a pad-only control (parent padded to 88 bytes with no behaviour change), and v001 could not isolate it.

## Continuation

Merge v003's single multiplicative hash with v002's lazy promotion into the new base, keep stride at 80, and verify it against both parents with repeated same-session runs. Next, probe a lazy-promotion window of `n/4` with a stop rule at the insertion region, then profile what remains above and inside the bucket. Drop miss memos, any other state added to skip cheap scans, and any writes on the get path. Keep the policy frozen except for the promotion window.
