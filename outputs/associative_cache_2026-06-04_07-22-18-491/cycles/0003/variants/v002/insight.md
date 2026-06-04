# Variant v002 Insight

## Helped

*(no dimensions helped)*

## Hurt

- hit-rate: replacing FIFO stash eviction with LFU-minimum selection degraded hit rate because items entering the stash are already the lowest-frequency items in primary (they were selected by the same LFU-weighted policy), so their `lfu_counter` values cluster near zero — the frequency signal within the stash is too compressed to discriminate usefully, and LFU ends up evicting arbitrary victims rather than the truly cold ones.

## Takeaway

The stash is a post-LFU residue: every item in it arrived there precisely because it had the lowest frequency in primary. Applying a second LFU pass on this already-filtered population finds no meaningful frequency gradient to exploit. FIFO's age-based ordering is better matched to the stash's role — it gives each item a fair temporal window to be promoted on re-access, and that window is what v001's hit-rate gain relied on. Frequency-based retention inside the stash is redundant with the promotion-on-hit mechanism that already rewards re-accessed items.
