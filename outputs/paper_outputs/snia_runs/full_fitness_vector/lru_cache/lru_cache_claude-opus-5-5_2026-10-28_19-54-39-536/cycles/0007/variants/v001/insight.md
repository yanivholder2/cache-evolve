# Variant v001 Insight

> Note: the overlay source files and manifest could not be found or listed from this session. This interpretation is based on the hypothesis (H01), both fitness vectors and the parent's insight. It assumes the overlay implements H01 as written.

## Helped

- per-op-time (get p99): Get p99 fell sharply and is now well below the parent's spike. Probation→protected promotion and protected-LRU demotion now run in an out-of-line `#[cold]` `promote`, and eviction runs in `evict_and_insert`. This was a layout-only change, with hit rate identical and the same insert-before-remove order in the table. So the parent's get-p99 rise was most likely a code-layout or inlining effect, or noise. It was probably not the hash-table side effect of deleted-slot markers that the parent insight suspected.

## Hurt

- per-op-time (get p50): The summary metric got worse, which is the opposite of what H01 predicted. Forcing `on_hit`, `unlink` and `push_back` to `#[inline(always)]` probably made the `get` body bigger. The `#[inline(never)]` call to `promote` also sits on a branch inside `get`, which can make the compiler set up registers or spill values even on the common protected-hit path. The result is a slower median, not a "small, tight body".

## Takeaway

Hit rate and metadata size match the parent exactly, and put moved only within noise, which confirms the change was layout-only. Splitting out cold code and forcing hot helpers inline trades get p50 for get p99. The median got worse while the tail recovered, so the get-p50 drift across cycles is really a layout effect, but forcing `#[inline(always)]` pushes it the wrong way. A follow-up should keep the cold `promote` and `evict_and_insert` split, which seems to be what fixed the tail. It should drop the `#[inline(always)]` hints and leave inlining to the compiler, so the `#[cold]` split can be tested on its own against get p50.
