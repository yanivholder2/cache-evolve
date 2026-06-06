# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Admitting high-frequency newcomers directly into the protected segment forces an immediate `maybe_demote_from_protected()` call, which evicts the current protected LRU resident earlier than the normal `get()`-driven promotion path would — the added churn displaces items that had already earned their protected position and outruns any gain from shortcutting probationary.

## Takeaway

Bypassing probationary at admission time for sketch-hot items proves counterproductive because the protected segment is a zero-sum space: every direct-to-protected insert immediately displaces an existing protected resident via demotion, whereas the normal flow (enter probationary, promote on first `get()`) defers that displacement until the item's hotness is reconfirmed by an actual access. The probationary gauntlet is not mere overhead for re-encountered hot items — it acts as a timing gate that prevents newly admitted items from cannibalising protected capacity before they prove continued demand on this workload.
