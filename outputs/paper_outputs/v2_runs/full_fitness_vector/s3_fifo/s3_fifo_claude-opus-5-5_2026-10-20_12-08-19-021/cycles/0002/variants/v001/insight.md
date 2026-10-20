# Variant v001 Insight

## Helped

(none. `hit-rate` stayed flat: the drop is within one ten-thousandth, which is noise, not the gain H01 predicted. `metadata-size` didn't move because neither `Entry<K,V>` nor `S3Fifo` changed. The diff is only the `tail.freq > 1` → `tail.freq >= 1` predicate plus a new unit test.)

## Hurt

- per-op-time (put): Under the looser predicate, small evictions that used to be a cheap ghost `push_front` become graduations. When main is full, each graduation runs `evict_main()`. If `grow_small` also fires, it runs a second fix-up `evict_main()`. Graduates also keep their `freq`, so main fills with more nonzero-freq entries, and each eviction has to rotate through more of them in the second-chance loop before it finds a victim.
- per-op-time (get): `get` has no direct link to the change. Its code path (a linear scan of small, then main) is unchanged. A plausible cause: once-reused keys now sit in main instead of being re-admitted from ghost, and more frequent `grow_small` makes small larger, so a hit has to scan further before it reaches its key. But v003's own insight called its low `get` latency unexplained, so part of this regression may just be the parent's number reverting to normal. Re-measure both variants before blaming the predicate.

## Takeaway

On workload-a, letting a key with a single re-access graduate (the paper's rule) adds nothing on top of v003's adaptive split. Under the old rule, those keys went to ghost and returned to main on their next access anyway, so the new rule mostly changes when they enter main, not whether they do. Meanwhile, more graduates arriving while main is full push `grow_small` to give slots back to small, which likely cancels any benefit. The only measurable effect is extra main-eviction work on `put`. So for this trace, the graduation threshold is not a useful lever when the split is adaptive. A future variant that wants this rule should log where `small_size` settles, to check whether the adaptation is absorbing the change.
