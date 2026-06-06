# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Doubling the ghost window (`ghost_size = main_size * 2`, applied as the new trim bound in `evict_small`) kept stale evicted keys around long enough that their delayed re-appearance still triggered a direct-to-main promotion, and these low-quality re-admissions displaced hotter resident entries instead of surfacing genuine reuse — pushing hit-rate just below the main-sized-ghost parent.

## Takeaway

On this workload the ghost depth was already past its useful horizon at `main_size`: remembering ~2× as many evicted keys does not capture longer-gap reuse, it manufactures false re-admissions because the unguarded `in_ghost` path treats any ghost hit — fresh or ancient — as confirmed-reuse. Ghost *sizing* alone is an inert-to-harmful lever here; without a freshness gate, a deeper ghost only pollutes main.
