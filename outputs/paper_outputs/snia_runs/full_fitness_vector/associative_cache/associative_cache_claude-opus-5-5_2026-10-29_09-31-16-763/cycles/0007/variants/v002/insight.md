# Variant v002 Insight

## Helped

- hit-rate: The gain shows up only in the web_0 131072-size cell. That is the cell the hypothesis aimed at. On buckets wider than 256 slots, `bip_mask` switches from 31 to 127, so scan keys reach the MRU end 4× less often and stay in the LIP quarter-depth zone, where they are evicted early. Both 16384-size cells match the parent exactly. This confirms that the narrow-bucket branch (`max_size <= BIP_WIDE_BUCKET`) keeps the parent's 1/32 behaviour bit-for-bit. The extended `fused_matches_reference_remove_insert` test (sizes 256, 300 and 512) checks that the fused rotation still follows the reference order on both sides of the threshold.

## Hurt

- hit-rate: hm_0 at 131072 dipped slightly. The hypothesis expected it to stay "roughly flat", and the dip is small enough to count as that. Still, it shows that hm_0's shifting working set on wide buckets used part of the 1/32 MRU admission rate, so at 1/128 a few of its keys now enter the protected region too late. The web_0 gain is only a little larger than this loss, so the net summary gain is small.
- per-op-time: Get and put both got slower at p50 and p99, but the diff barely explains it. The get path (`get_position_tagged`, `get_from_position`) is byte-for-byte the parent's. On put, the only change is one compare on `max_size`, which is already loaded, plus an `&` that replaces a `%` by a power of two (the compiler already turns that `%` into the same AND). A get-p50 regression on untouched code points to measurement drift or a code-layout/inlining shift, not to the width-dependent period. This needs a re-run before anyone attributes it to the change.

## Takeaway

A static, width-keyed BIP period does what the cycle-0002 data predicted. On wide buckets it moves scan-heavy web_0 toward LIP and leaves narrow buckets exactly as they were. But the hm_0 cost on the same wide buckets cancels most of the gain, so the 1/32 vs 1/128 choice on 1024-slot buckets is close to a zero-sum trade between web_0 and hm_0, not a free win. The per-op-time slowdown cannot come from a one-compare change and should be re-measured before this variant is judged on speed.
