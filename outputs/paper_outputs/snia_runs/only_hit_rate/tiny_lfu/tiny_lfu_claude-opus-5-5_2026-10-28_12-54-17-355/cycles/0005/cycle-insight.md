# Cycle 0005 Insight

## Cohort outcome

All three variants branched from cycle 0004's adaptive window + adaptive protected split. They tried a sketch fix that counts each miss-then-insert once (v001), an ARC-style ghost-list controller in place of the window climber (v002), and taking the coldest of the 4 oldest probation entries as the eviction victim (v003). v003 won. It is the first clear, regression-free gain in several cycles, and it gained on web_0 at both sizes. v001 traded web_0 small against hm_0 small, and v002 regressed in every cell.

## Patterns

- Every hypothesis aimed at hm_0, and none moved it in the way it predicted. Each one's stated mechanism was refuted. The gains that did land came on web_0 and through frequency-based selection: a sharper admission contest in v003 and less collision noise in the small sketch in v001. Changes that make the cache better at picking by frequency keep paying off. Attempts to rescue recency-driven entries keep missing.
- The admission/eviction structure is still where progress comes from, and feedback-loop redesign is still where it fails. v002 makes five cycles in a row of failed window-controller changes. Ghost lists fit W-TinyLFU poorly: the frequency filter, not the window size, produces most window rejections, so the two ghost lists fill very unevenly and the controller ratchets the window down to its minimum. Applying the controller on every event also sent traffic through the eviction paths that skip the admission contest.
- Position matters more than choice for hm_0. Fresh admissions land at the MRU end of probation, but victim sampling only looks at the LRU end. v001 also showed that hm_0 small had been relying on an accidental bias: the double count gave a frequency bonus to keys that returned after eviction. Taken together, hm_0 seems to want an *explicit* recency or re-entry signal, not a different choice among the oldest few entries.
- Single-count sketch accounting is the correct base, even though it lost on hm_0 small.
- Metadata-size and per-op-time are still unmeasured, now five cycles in a row. v003 adds a few sketch estimates per full-cache insert. v002's ghost sets would have added a lot of metadata.

## Open questions

- Does v003's victim sampling combine with v001's single-count fix? The two gains come through different routes (contest bar and collision noise).
- Does a larger victim sample keep strengthening web_0, and at what point do the extra per-op estimates cost too much?
- Can hm_0 be helped on purpose by giving a doorkeeper/ghost bonus to recently evicted keys when they come back, or by changing where new admissions land in probation or how long they stay there?
- Could a ghost signal work with balanced inputs (only size-caused window evictions, normalised per list) applied in batches at period boundaries with small steps? Or should controller work stop entirely?

## Continuation

Build on v003, the sampled probation victim. First, layer v001's single-count sketch fix onto it, and sweep the victim sample size as a separate step. For hm_0, try an explicit re-entry/recency bonus or a change to where admissions land in probation. Don't try any more victim-choice tweaks at the LRU end. Drop per-event ghost-list window controllers, and freeze the window/protected climbers unless a balanced, batched signal is proposed. Measure metadata-size and per-op-time.
