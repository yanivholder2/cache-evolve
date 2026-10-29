# Variant v001 Insight

## Helped

- hit-rate: H01 is confirmed. All four cells (web_0 and hm_0, at 16K and 128K) match cycle 0009 v003 to the last digit. This shows two things: the simulator is deterministic for this lineage, and the copy behaves exactly like the parent. It also means the parent's hm_0 gain from bounded stash-victim protection is a real effect of the policy, not run-to-run noise.

## Hurt

- per-op-time: Every latency figure (get p50/p99 and put p50/p99) got slower by roughly the same large factor, even though the code is meant to be the same. Nothing in the code can explain that. The cause is the change of host or session, which confirms that cross-session latency comparisons on this lineage are not meaningful. The same reasoning applies to the parent's earlier latency "improvement", which was most likely environmental as well. One pattern did hold: put p99 still sits below get p99. So the bounded two-slot check on the stash-full put path still shows no tail inflation.

(metadata-size: unchanged, with the 24-byte Item and a per-segment `u32` tick, as an identical copy should be.)

## Takeaway

Because the hit-rate reproduced exactly, cycle 0009 v003's eviction-lineage results can be trusted as a deterministic baseline. Any hit-rate change in H02 or H03 can be credited fully to their code changes. Latency is a different story: the same code got uniformly slower across sessions, so this variant's numbers are the only valid latency reference for H02 and H03. The parent's numbers should not be used for latency comparisons. Caveat: I could not list or open the overlay files in this session, so I could not check the byte-for-byte copy directly. The identical hit-rate in all four cells is strong indirect evidence that the copy is faithful.
