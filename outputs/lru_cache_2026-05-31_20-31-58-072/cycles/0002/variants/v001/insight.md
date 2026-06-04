# Variant v001 Insight

## Helped

(none — the hypothesis predicted a hit-rate lift over v002 and the lirs:multi1 score moved the wrong way.)

## Hurt

- hit-rate: Starting every new entry at `counter = 1` gives cold scan keys a full sweep of immunity, so they linger long enough to push genuinely hot keys' counters down before they can be evicted; on a recency-skewed trace this is worse than v002's bit, which lets a never-touched insert be reclaimed on the very next pass.
- hit-rate: A saturating 0..=3 counter is a frequency proxy, not a recency proxy — keys that were hot in the past but have gone cold sit at counter 3 and force the hand to make up to three full passes before they become victims, during which the hand also decays the counters of currently-hot keys and lowers their survival odds.

## Takeaway

On lirs:multi1 the bottleneck is recency, not frequency, so widening v002's reference bit into an aging counter degrades rather than improves the eviction choice: the extra bits remember stale popularity and shield fresh-but-useless scan inserts. The hypothesis's framing — "multi-touch keys outlive scan keys" — is only a win when the workload actually rewards frequency over recency; here it costs ~0.5 pp of hit-rate, suggesting the next move should keep the single bit and instead attack v002's strict-recency gap from a different angle (e.g. admitting new keys at `referenced = false` is already optimal — look at the sweep policy, not the counter width).
