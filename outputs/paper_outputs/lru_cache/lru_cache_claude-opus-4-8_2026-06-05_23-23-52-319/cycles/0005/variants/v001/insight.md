# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Swapping the protected-tier victim from `prot_head` (LRU) to the minimum-`freq` node lets `enforce_protected_budget` demote keys from anywhere in the list, including ones touched more recently than the true LRU, pushing them back into the churning probationary segment and exposing them to eviction sooner.

## Takeaway

On this reuse-heavy LIRS trace, frequency is *not* the untapped axis the hypothesis assumed: at a near-full 95% protected segment most nodes sit at low, tied `freq` values, so the LFU scan mostly tracks recency anyway but breaks toward demoting recently-promoted hot keys instead of the genuinely cold LRU. Recency at the demotion point beats access frequency here — the retention victim selector should stay LRU-based.
