# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Raising the second-chance gate from `freq > 1` to `freq > 2` in `evict_main()` causes items that enter main at freq==2 — the common starting state after eager promotion from small — to be evicted on their very first pass to the tail with no recycle, stripping protection from recently-promoted items that still have residual demand.

## Takeaway

The cycle/0006 win (freq > 0 → freq > 1) worked because it evicted truly cold one-time-access items from main. Extending the same logic to freq > 2 misfires because eager promotion (introduced in cycle/0007) deposits items into main already carrying freq==2; the tighter threshold turns what was a second-chance reprieve into an immediate eviction for those freshly-promoted entries. The interaction between the promotion entry-point frequency and the eviction threshold means the "medium-demand tier" hypothesis doesn't hold here — freq==2 residents in main are recently-arrived hot items, not stale squatters.
