# Variant v002 Insight

## Hurt

- hit-rate: Overly aggressive frequency boost to ghost hits reduces overall hit rate by crowding the main queue with items that don't justify their elevated protection status.

## Takeaway

Assigning initial frequency 2 to items re-admitted from the ghost queue creates too much protection, preventing faster eviction of marginally-useful ghost hits and reducing space for competing items. While the intuition that ghost hits represent proven reusable items is sound, the blanket boost appears to violate S3-FIFO's delicate frequency-based balance, where items should earn their protection through actual reuse patterns rather than a priori membership in the ghost queue.
