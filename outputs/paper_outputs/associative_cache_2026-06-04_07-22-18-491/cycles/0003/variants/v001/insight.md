# Variant v001 Insight

## Helped

*(no dimensions helped)*

## Hurt

- hit-rate: doubling the stash capacity to `bucket_size / 2` extended the survival window for items that the LFU-weighted policy had correctly identified as low-frequency, allowing genuine cold items to occupy stash slots long enough to be accidentally re-accessed and promoted back into primary, displacing hotter residents in the process.

## Takeaway

The `bucket_size / 4` stash cap in the parent was not a limitation — it was a filter. Because LFU-weighted eviction selects the lowest-frequency items to enter the stash, a small cap naturally ages out items with near-zero reuse before they can cause spurious promotions. Doubling the cap breaks this self-cleaning property: cold items linger twice as long in the stash, raising the probability that a one-off re-access promotes them into primary and triggers a cascade that evicts a genuinely medium-frequency item. The workload's medium-frequency working set fits within the smaller stash window; extra capacity only accumulates noise.
