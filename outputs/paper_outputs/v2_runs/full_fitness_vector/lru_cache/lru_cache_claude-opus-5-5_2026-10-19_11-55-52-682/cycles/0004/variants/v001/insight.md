# Variant v001 Insight

## Helped

(None. No dimension improved. The `get` path was left untouched, as planned, so its median latency held. But holding steady is not a gain.)

## Hurt

- hit-rate: The ghost table was meant to raise hit-rate, but it lowered it. In plain SIEVE, every new key already starts at the tail. It gets a full sweep of residence before the hand reaches it. Pre-marking a ghost-hit key `visited` gives it a second sweep. A ghost hit only proves that the key's reuse distance exceeded one residence. On workload-a, those returning keys usually don't come back within the extra pass. Meanwhile, the hand clears their bit and evicts some other unvisited node instead, often a key that would have hit sooner. Unlike S3-FIFO, there is no probationary small queue for the ghost signal to bypass, so the signal can only displace residents. It cannot filter one-hit wonders.
- per-op-time: The headline (`get` p50) is unchanged, but `put` got slower, mostly at the tail. Every miss-insert now pays for two extra `hash_one` calls: one on the evicted key and one on the incoming key in `ghost_take`. That comes on top of the hashes `map.get` and `map.insert` already do. Each insert also makes up to two random reads and writes into a `Vec<u32>` sized to the cache's power-of-two capacity, which adds cache-line misses on the eviction path.
- metadata-size: The per-node summary did not move, because `Node` stays 24 bytes as the hypothesis required. But the `LruCache` struct grew by the `ghost` Vec header and `ghost_mask`. The parent's padding trick no longer hides the extra fields. There is also a heap table of 4 bytes per power-of-two capacity slot, which the per-node summary does not count. So the hypothesis did not deliver its claim of "without growing" metadata. It only moved the growth to where the summary doesn't measure it.

## Takeaway

On singly-linked SIEVE, a ghost hit that only sets the initial `visited` bit just lengthens the residence of keys that have already shown their reuse distance exceeds the queue. On workload-a that crowds out better residents, so hit-rate drops, and the extra hashing on every miss slows `put`. The S3-FIFO ghost idea only pays off when there is a probationary queue for ghost hits to skip. Grafting it onto a single SIEVE queue, the smallest possible change, removes exactly the part that made it work.
