# Variant v003 Insight

## Helped

(No measured dimension moved. hit-rate matches the parent to every recorded digit. That is the result H03 predicted, since the aim was to hold hit rate while making the ghost metadata smaller.)

## Hurt

- hit-rate: Nothing measurable was lost. Swapping exact `Option<K>` matches for non-zero 16-bit fingerprints can only cause harm through a false match within one bucket's 4-entry ring, which moves a newcomer from index 1 to index 3. On this trace such false matches were too rare, or too harmless, to change the result.
- (Not recorded: per-op-time. Each miss on the ClassicLRU path now runs the key through the FNV-1a + fmix64 hash once for the ghost lookup, and again for every eviction it records. The parent only did a 4-way equality scan. A small per-miss cost is likely but has not been confirmed. metadata-size was not recorded either. For 8-byte keys, the ghost ring should shrink from four 16-byte `Option<K>` slots to four `u16`s, the 64-to-8-byte cut H03 claimed. That saving is still unmeasured, and it grows with wider `K`.)

## Takeaway

On the 1/8-depth LRU line, the ghost ring is also used only as a membership signal, so exact key identity is not needed. Four 16-bit fingerprints per bucket keep all of the parent's ghost-hit benefit. This rules out H03's "different representations per line" alternative: the CLOCK and LRU lines can share the cycle 0007 fingerprint ring, even though the cross-cycle notes say they capture different reuse-distance populations. The fingerprinting is now "free" on hit rate, so the open question is the trade between metadata-size and per-op-time. A future evaluation must record both before this line can be fairly ranked on cost.
