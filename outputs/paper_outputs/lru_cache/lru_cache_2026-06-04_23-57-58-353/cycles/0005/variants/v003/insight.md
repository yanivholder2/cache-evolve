# Variant v003 Insight

## Helped

## Hurt

- hit-rate: Requiring a second probationary access before promotion forces legitimately hot entries to spend an additional residency window in probationary, where they remain evictable by the H03 scan, and enough such entries are lost during that extra exposure to outweigh any reduction in one-hit-wonder promotion.

## Takeaway

The two-hit gate was designed to filter transient entries that happen to be accessed once while still in probationary, but on workload-a the H03 frequency-aware eviction scan already provides sufficient cold-traffic filtering — entries that survive in probationary are generally worth promoting immediately. The gate's cost (delayed protection for hot keys) proved larger than its benefit, indicating the workload's one-hit-wonder rate is too low to justify the added exposure risk.
