# Variant v001 Insight

## Helped

- None improved over the parent on comparable data. (hit_rate was measured on
  the `lirs:multi1` scan/loop trace this hypothesis targeted, but the parent's
  vector has no entry for that trace — only the caffeine assoc/dash micro
  configs — so the frequency-gated promotion's hit_rate effect cannot be
  attributed from what is available here.)

## Hurt

- per_op_time: every hit now bumps `lfu_counter` (in `ClassicLRU::get_from_position`
  and `DashBucket::remove_at`), and the target-hit promotion check scans the whole
  bucket for a strict-max frequency leader (O(n)) instead of reading the fixed MRU
  slot (O(1)), so get-path latency rose.

## Takeaway

Routing frequency-gated promotion through the `EvictionPolicy` seam, this variant
gates the climb into the protected probing tier on a strict *pre-hit* frequency
maximum where ties never promote — a very conservative bucket-movement rule: on a
loop workload whose residents share similar access counts, the strict-max test
seldom fires, so few keys earn promotion and the hot tier stays under-filled. The
frequency signal itself was free (it reused Item's existing `lfu_counter`, so
metadata-size held flat), but trading the O(1) positional MRU promotion check for
an O(n) per-hit strict-max scan is a measurable get-path tax.
