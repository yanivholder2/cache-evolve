# Variant v003 Insight

## Helped

- hit-rate (web_0 @ 131072): This cell is the one H03 was aimed at, and it gains the most. Conservative update leaves a key's counters alone when they sit above its current minimum. That stops cold keys from inflating the counters they share with a hot key in the same 64-byte block, so `estimate_hashed` gives less inflated frequencies for victims and candidates. The admit-or-drop comparison at the head of the LRU then rejects fewer of the genuinely reused keys a large cache can hold. Since `SAMPLE_FACTOR` and the reset period were left alone, this supports v002's "correlated in-block collision" explanation for the slip over the aging-period explanation.
- hit-rate (other cells): web_0 @ 16384 and both hm_0 sizes also move up slightly rather than holding flat. Less overestimation helps admission decisions across all sizes. It also slows how fast collided nibbles reach the 4-bit ceiling of 15, so the min-of-four estimate keeps ranking power for longer between halvings.

## Hurt

- per-op-time: Get and put latency both get substantially worse at the median and the tail, and the tail is hit hardest. `increment_hashed` runs on every `get` and `put`. It is now two passes: it fills `pos`/`vals` arrays, reduces them to a minimum, then does a second loop with a data-dependent `vals[i] != min` branch and a re-read-then-compare guard on each nibble. Which rows equal the minimum changes from key to key, so those branches mispredict often. The re-read also adds a load-after-store dependency on `block[w]` for rows that share a word. The parent did a single branch-predictable pass ("not saturated", almost always true) with independent adds. So the hypothesis's argument that "no extra cache misses" means no cost was wrong. The cost is in control flow and dependency chains, not memory.

## Takeaway

Conservative update is a real accuracy fix for the blocked sketch. It recovers the large-cache web_0 cell and lifts the rest, which shows v002's slip came from correlated overestimation rather than the aging period. But implementing it as a branchy two-pass read-then-conditionally-write loop throws away much of the latency v002 gained from the single-cache-line layout. The next step should keep the conservative-update semantics and make the second pass branchless. For example, compute a per-row increment mask (`c_i == min`) and apply it with masked adds to the packed words, instead of revisiting `SAMPLE_FACTOR`.
