# Cycle 0005 Insight

## Cohort outcome

Variants probed memory layout for get (a contiguous tag-first scan, 64-byte bucket alignment) and nudged the insertion point from len/4 to len/3. None improved get. v001 and v002 were Winners, but both on thin evidence, since their own insights report a hit-rate loss and a slower put (v001), or a hit-rate loss with an unexplained put gain (v002).

## Patterns

- Get latency again did not move, now across five cycles. Arithmetic, hashing, item movement and layout (tags, alignment) are all ruled out. Buckets are probably small and cache-resident, so get is likely dominated by something outside bucket memory access.
- Layout changes that must stay in sync with promotions and swaps (parallel tags) raise write cost and risk drifting from the parent's policy. Pure allocation-placement changes (alignment) are safe, but their benefit showed up only on put tail latency.
- Insertion depth is a free hit-rate knob. Moving from len/4 toward the protected end lowered hit rate, which fits the earlier saturation finding that len/4 or lower is the useful range.
- Put-latency gains without a mechanistic reason (v002) are the same noise pattern seen in cycles 0001 and 0002.
- No variant changed metadata size.

## Open questions

- Are the v001 and v002 Winner statuses real, or are they artifacts of noisy put latency and a flat get?
- Does the optimum insertion point lie below len/4, such as len/5 or len/8?
- Where does get time go if not memory layout, hashing or reordering? It may be harness overhead, dispatch or the key and value path.
- Does cache-line alignment's put-tail gain repeat, and does it stack with swap-based insertion?
- Do any gains hold on a second workload?

## Continuation

Stop attempting get-latency gains through bucket layout, tags or hash arithmetic; the evidence says they do not work. Keep the swap-based len/4 insertion as the policy baseline, try slightly lower insertion points, and consider adopting plain 64-byte alignment as a cheap put-tail improvement after re-checking it. Add a second workload to validate any claimed Winner, and discount Winners whose own insight shows a loss.
