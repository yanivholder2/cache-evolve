# Cycle 0004 Insight

## Cohort outcome

All three variants branched from singly-linked SIEVE (cycle 0003 v001). v001 grafted a ghost table of evicted keys onto the single queue. v002 swapped SipHash for an Fx-style hasher with `u32` map values. v003 moved the link and visited bit into a packed `u32` side array. All three are listed as Winners, but the outcomes split. v002 is the clear gain: faster get and put, hit-rate unchanged, and a smaller cache struct. v003 is a metadata-only gain with the same eviction order. v001 regressed on hit-rate and put latency.

## Patterns

- Representation-only changes stayed safe for the third cycle running. Both v002 and v003 kept eviction order bit-for-bit. The only policy change, v001, cost hit-rate.
- This cycle found the latency lever: hashing. Cycle 0003 already showed node layout no longer moves speed. v002 then moved get p50 for the first time in several cycles, and put p99 improved even more. This supports cycle 0003's view that the put tail comes from map work on the miss path, not from the hand. v003's denser sweep was roughly neutral on speed, which confirms that layout is now only a memory lever.
- A ghost signal needs a probationary queue to bypass. On a single SIEVE queue, a ghost hit can only extend the stay of keys whose reuse distance already exceeds the queue. Those keys push out better residents. v001 also added hashes on every miss, so the "outside-the-queue signal" question is still open. Only the minimal graft has been refuted.
- The metadata metric is drifting out of step with the real footprint. v001 put its growth in a heap table, and v003 put its policy state in a side array the per-node summary doesn't count. Read node-size wins with care unless side arrays are added in per entry.

## Open questions

- Does a real S3-FIFO split (small probationary FIFO + main SIEVE + ghost) gain hit-rate where the single-queue graft lost it?
- How much of put p99 remains after cutting probes per miss (entry-style insert, reusing one hash across the probe, remove and insert)?
- Do v002's hasher and v003's side array stack cleanly, and does the extra memory touch on each hit matter once hashing is cheap?
- SIEVE's lead at other cache sizes or traces is still unverified after three cycles.

## Continuation

Use v002 (Fx-hashed SIEVE) as the parent and fold in v003's side-array layout as a low-risk combination. For latency, cut the number of map probes on the miss path. Do not do further node-shape work for speed. Drop the single-queue ghost graft. If ghosts come back, they must come with a small probationary queue (S3-FIFO-style). Count side-array bytes per entry when judging metadata, and finally add a second cache size or trace.
