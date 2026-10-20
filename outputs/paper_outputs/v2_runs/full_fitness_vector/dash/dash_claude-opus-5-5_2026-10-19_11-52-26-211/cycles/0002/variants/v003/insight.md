# Variant v003 Insight

## Helped

- hit-rate: `DashSegment::put` now inserts fresh keys at the stash's LRU end (`FRESH_PUT_STASH_POS = Front`), while demotees still enter at Back. So when a stash bucket overflows, the newest never-read arrival is evicted first, and items that have already been reused survive. Hit rate rose above the parent's, clearing the hypothesis's 0.684 falsification bar. That means on workload-a, most keys that are ever reused get their second access before their stash bucket next overflows. Losing the keys with longer first-reuse distances cost less than protecting demotees gained.

## Hurt

- per-op-time: The headline `get` p50 is essentially unchanged, since `promotion_get` was not touched. But `put` p50 went up, which fits the new Front insert: every fresh put now shifts the stash bucket's contents, where the parent's Back insert simply appended. The v001 insight predicted exactly this. The movements in `get` p99 and `put` p99, in opposite directions, have no code-level cause in this diff, so they are most likely run-to-run noise.

(metadata-size did not change: only an insert position changed. No fields or types were added, and `Item` stays at 40 bytes.)

## Takeaway

Making the stash a scan-resistant probationary queue pays off on top of v001's SLRU demotion. v001 put demotees above fresh arrivals by sending them to Back. v003 gets the same ordering from the other side by sending fresh arrivals to Front, and the gains add up. The two changes together show that this workload rewards ranking by proven reuse over pure recency inside the stash. The change was isolated to the fresh-insert position, with `DEMOTE_TO_STASH_POS` and promotion placement left unchanged. So the remaining untested lever is still where `get` places promoted items (Front vs Back). Note that a Front insert adds a small cost to `put` from shifting the bucket.
