# Baseline Insight

N-way associative cache: keys are hashed to a fixed bucket, and each bucket is a
fixed-capacity ordered collection that evicts per its eviction policy
(ClassicLRU baseline). No probing, stash, or inter-bucket migration — a key
lives only in the single bucket it hashes to.

This is the seed baseline: the starting point every cycle-1 cohort is proposing
improvements over. Measured fitness lives in `eval/fitness-vector.json`.
