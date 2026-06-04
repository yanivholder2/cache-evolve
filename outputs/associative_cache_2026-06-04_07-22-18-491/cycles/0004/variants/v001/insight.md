# Variant v001 Insight

## Helped

*(no dimensions helped)*

## Hurt

- hit-rate: raising the promotion credit from +2 to +3 kept stash-promoted items in primary too long, displacing genuinely high-frequency occupants and causing them to miss before the promoted items' inflated counters could be earned down.

## Takeaway

The promotion credit lever is not monotone: +3 overshoots the optimum that +2 established. With a credit of +3, promoted items hold their primary slot through one extra eviction round beyond what their true access frequency warrants, crowding out items that accumulated their LFU counts legitimately. This confirms +2 as the local optimum for this workload — the credit needs to be large enough to survive a single cold-burst displacement, but not so large that it artificially outbids recurring items for primary slots.
