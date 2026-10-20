# Cross-Cycle Insights

## Recurring wins

- Moving away from strict LRU recency order has improved hit-rate on workload-a in both cycles. Second-touch promotion, second-chance bits and quick demotion of new keys all beat plain LRU.
- A hit path with no relinking (CLOCK, GCLOCK, SIEVE) keeps get latency low in both cycles. Hand-based designs have the cheapest get.
- Node layout is the dependable lever for metadata-size. Fewer links or narrower `u32` index links shrink the node and usually help speed through better locality. Adding small policy state (`u8` freq, `bool` visited) in padding bytes costs no metadata.
- Speed gains have come from memory locality (compact nodes, fewer pointer writes) rather than shorter code paths.
- Simple single-queue designs can match or beat multi-segment ones. SIEVE (one queue, one bit) has the best result so far and improved on every dimension.

## Recurring failures

- Changing only where plain LRU inserts or orders keys has run out of room. It was dropped after cycle 0001.
- Scan defenses stacked on each other don't add up. BIP head insertion inside SLRU probation lost hit-rate because it shrank the window in which new keys can earn a second touch.
- Deeper per-slot credit on a hand-based policy (GCLOCK 2-bit) buys hit-rate with longer sweeps, which hurts put p99.
- The cost of hand sweeps keeps appearing in the put tail: it was a concern for CLOCK in cycle 0001 and was confirmed for GCLOCK and SIEVE in cycle 0002.

## Open questions

- Can the sweep be bounded without losing the frequency or visited signal? Options include a capped number of steps per insert, a smaller `FREQ_MAX` or a faster decrement.
- Can a SIEVE/GCLOCK hybrid (small visited counter) gain hit-rate without GCLOCK's tail cost?
- Do the rankings hold at other cache sizes or on other traces? Every result so far comes from workload-a at one size (2048), so the leaders may be overfit.
- Would changing the SLRU probation/protected split or adding an admission filter beat SLRU's current split, given that changing the insertion position did not?
- Some small speed differences (for example BIP's in cycle 0001) may be benchmark noise. This has not been checked.

## Per-policy notes

- **LRU (baseline):** a weak hit-rate reference on workload-a. Further tweaks to its ordering alone are not worth pursuing.
- **SLRU:** a solid winner in cycle 0001 thanks to the probation/protected split and `u32` links. Adding BIP inside probation hurt hit-rate. If revisited, tune the segment sizes or admission rather than the insertion position.
- **CLOCK:** gained hit-rate unexpectedly, smallest node, relink-free hits. Its sweep cost is the known weak spot.
- **GCLOCK (2-bit):** more hit-rate than CLOCK but clearly worse put p99 from longer sweeps. Useful mainly as a reference for what frequency credit costs.
- **BIP:** a winner as a standalone change to LRU (loop resistance), with metadata flat or slightly up. It is harmful when combined with SLRU probation.
- **SIEVE:** the current main parent: best hit-rate and better on all dimensions, though by a narrow margin. Its smallest gain is at put p99 because of long runs of visited nodes, so the next target is bounding the sweep.
