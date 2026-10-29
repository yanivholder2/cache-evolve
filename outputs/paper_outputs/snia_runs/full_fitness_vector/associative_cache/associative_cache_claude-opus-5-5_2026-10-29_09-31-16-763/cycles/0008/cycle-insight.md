# Cycle 0008 Insight

## Cohort outcome

All three variants branched from cycle 0007/v003, the clock-free fused-rotation base on byte tags. v001 hand-wrote the hit-path MRU shift with `ptr::copy`. v002 ported width-keyed BIP to this base at 1/64 on wide buckets. v003 replaced the byte tag scan with a SWAR scan. v002 and v003 are Winners. v002 gave a small net hit-rate gain. v003 is a Winner only because its invariants held: get and put both got clearly slower. Neither speed hypothesis produced a speed win.

## Patterns

- Both speed rewrites backfired at p50 on get and put. These are large, same-direction moves, not tail noise. In both cases the rewrite swapped a simple loop the compiler handles well for "clever" code. The variable-length `ptr::copy` probably became an out-of-line `memmove`, and SWAR added fixed per-chunk work plus a branchy mask loop. The put win from cycle 0006 does not generalise. Hand-written moves pay off only when they remove whole memmoves, not when they replace a short inline move.
- Counting compares stopped working as the cost model. The tag pre-filter won in cycle 0005 by cutting full-key compares. Making the byte scan itself wider did not help, which suggests typical scans are short or already vectorised. The remaining miss-path cost is probably not in the tag scan.
- Two caveats on the evidence. The v001 and v003 readings were written without access to the diffs or the generated assembly, so the proposed mechanisms (memmove call, de-inlining) are inferences. And get slowed in the hit-rate-only v002 too, so part of every per-op move this cycle may still be drift.
- The wide-bucket BIP probe is closed. Going from 1/32 to 1/64 to 1/128 moves the web_0 gain and the hm_0 loss roughly in proportion, so there is no sweet spot. The hypothesis's own stop rule fires. Width-keyed masking is still worth keeping as a zero-metadata way to split policy by bucket size: narrow cells stayed bit-identical again.
- Hit-rate and metadata-size invariants held exactly for all three variants. Mechanical rewrites remain semantically safe, so the risk is now entirely in per-op time.

## Open questions

- Does v001's slowdown come from a `memmove` call or de-inlining? Would a fixed-width or const-generic copy, or a fused swap loop, fix it? Is the parent's hit path already near-optimal?
- How far does a typical tag scan actually go before it matches or ends? If the answer is "short", where does the rest of the miss-path time go: the key compare, the rotation, or the bucket indexing and hashing?
- How much of this cycle's latency movement survives repeated same-session runs against the parent?

## Continuation

Adopt v002 (width-keyed BIP, 1/64 wide) as the new base and freeze the policy. Stop tuning the BIP period. Drop SWAR and other hand-widened tag scans. Drop variable-length `ptr::copy` on the hit path, unless an assembly check shows a fixed-width copy would stay inline. The next speed hypotheses should begin by locating the real cost: profile scan length, check codegen, and look at hash/indexing cost. Judge them only with repeated same-session runs against the parent.
