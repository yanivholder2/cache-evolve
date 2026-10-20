# Variant v001 Insight

## Helped

- per-op-time: put and get tail latencies (p99) improved slightly versus the parent, likely noise or a marginally simpler swap pattern; the median get cost was essentially unchanged, since the halving jump is still one swap.

## Hurt

- hit-rate: jumping hits to `i - max(1, i/2)` lets a single reuse push an entry deep into the protected region, so one-off or scan-adjacent reuses displace entries that had earned their positions gradually. This lowered hit-rate on the scan-heavy trace instead of raising it.

## Takeaway

Faster hit promotion did not beat transpose-by-one on this workload. Gradual one-step promotion works as a frequency filter, and a geometric jump removes that filter. Metadata-size was unaffected because the change adds no state. Promotion aggressiveness looks like a trade-off that the parent had already tuned well.
