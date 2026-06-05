# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Gating target→probing promotion on `lfu_counter >= 2` delayed genuinely hot items from reaching the more eviction-resistant probing tier, causing them to linger in the larger target pool where they compete with more entries and can be displaced before earning their second qualifying hit.

## Takeaway

Requiring two hottest-hits before promoting to probing was expected to reduce probing pollution from one-time scans, but on this workload the scan pollution was not a meaningful source of waste — probing slots were already being used productively. The gate's main effect was to slow the pipeline for items that are recurrently accessed, keeping them in the target tier one extra access cycle longer than necessary and slightly lowering the steady-state hit rate. The `lfu_counter` field is available and zero-cost, but the threshold of 2 is too conservative here; a threshold of 1 (i.e., the parent's unconditional promotion on first hottest hit) already performs better for this access pattern.
