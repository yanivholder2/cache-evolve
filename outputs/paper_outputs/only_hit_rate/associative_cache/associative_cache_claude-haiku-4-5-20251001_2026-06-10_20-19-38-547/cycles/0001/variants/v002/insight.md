# Variant v002 Insight

## Helped

(no improvements)

## Hurt

(no regressions)

## Takeaway

H02 hypothesized that probing neighboring buckets to place evicted items—rather than discarding them—would reduce collision load and improve hit rate through better item distribution. However, the cache's **slot_legality invariant** requires every key to remain in the exact bucket it hashes to, making inter-bucket item migration impossible without violating the architectural contract. The variant achieved baseline hit rate (0.6611) by reverting to the standard single-bucket eviction policy, demonstrating that cuckoo hashing-style probing cannot be applied here without first redesigning the cache's hash-to-bucket commitment.

