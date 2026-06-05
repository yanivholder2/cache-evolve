# Variant v001 Insight

## Helped

- hit-rate: the per-bucket FIFO stash gave a second chance to LFU-evicted items before they were permanently discarded, recovering medium-frequency keys that the LFU-weighted ClassicLRU had displaced during cold-burst insertions but that were re-accessed before the stash aged them out.

## Hurt

*(no dimensions hurt)*

## Takeaway

Combining the stash with LFU-weighted eviction is mutually reinforcing: LFU selects the genuinely least-accessed items to evict into the stash, so the stash population is already pre-filtered toward items with some reuse history rather than pure cold items. This means stash promotions are productive far more often than a naive second-chance mechanism would be, and the small stash cap (bucket_size / 4) is sufficient to capture the recency signal without cannibalising primary capacity.
