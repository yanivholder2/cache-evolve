# Variant v001 Insight

## Helped

- hit-rate (web_0, both sizes): once main is full, a small tail with only one reuse now goes to ghost instead of being promoted. That removes the extra `evict_main` pass per exactly-twice-accessed key that the parent insight blamed for wearing down hot entries' second-chance credit, so the parent's web_0 loss is undone.
- hit-rate (hm_0, 131072): the large cache also comes out ahead of the parent. So even on hm_0, some single-reuse promotions into a full main cost more in main pollution than they saved in avoided ghost misses. Routing those keys through ghost (re-admitted to main on their next access) was the cheaper path.
- hit-rate (summary): web_0 recovers and hm_0 at the large size gains slightly, so the aggregate improves on the parent.

## Hurt

- hit-rate (hm_0, 16384): this is the one cell that slips below the parent. With a small cache, main fills almost immediately, so the freq >= 1 path is live only during a brief warm-up. Here the variant behaves like the freq >= 2 rule, and hm_0's few-access keys go back to taking the extra ghost-miss detour that v001 had removed.

## Takeaway

Main never shrinks in this implementation: `evict_main` evicts one and the graduate immediately refills the slot. So "main has free capacity" holds only during cold start, and this variant is really the baseline freq >= 2 graduation rule plus the inherited freq reset, with a warm-up exception. The improvement over the parent therefore shows that the threshold of 1 was a net cost under steady-state pressure on web_0 and on hm_0 at the large size. Its benefit survives only on hm_0 at the small size, and even there the occupancy gate gives it up. If a looser threshold is worth keeping, it needs a signal that keeps changing in steady state, such as the small queue's recent reuse rate or the ghost hit rate. Main occupancy saturates and stays fixed, so it can't be that signal. Only hit-rate was measured. The `Entry` layout is unchanged, so metadata-size should be too; per-op-time wasn't measured.
