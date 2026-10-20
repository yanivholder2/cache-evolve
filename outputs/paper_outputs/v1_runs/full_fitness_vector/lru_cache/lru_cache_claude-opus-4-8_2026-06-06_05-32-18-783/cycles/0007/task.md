# Task: cycle-insight-writer

Cycle: 0007
Output dir: /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0007

## Winners

- v002
- v001

## Variant insights

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0007/variants/v001/insight.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0007/variants/v002/insight.md

## Hypotheses (this cycle)

- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0007/hypotheses/H01.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0007/hypotheses/H02.md
- /home/yaniv/projects/cache-evolve/outputs/lru_cache_2026-06-06_05-32-18-783/cycles/0007/hypotheses/H03.md

## Prior cycles digest

### Cycle 0004

# Cycle 0004 Insight

## Cohort outcome

This cycle finally cashed in two long-open levers and saw them both land
cleanly. The metadata fold (v001) re-encoded the SLRU champion's recency
links as `u32` sentinels and packed the segment flag into the link's high
bit, crossing into the 24-byte node class while holding the hit-rate
record and even shaving latency. The gentle ghost-admission rework (v002)
gated re-admission on repeated evidence and routed single hits to
probation, recovering the reuse-distance signal that overshot in Cycle
0003 and pushing hit-rate past the record — also without a latency tax.
Both were Winners. The raw protected-cap widening (v003) lifted hit-rate
but paid per-op latency, and lost.

## Patterns

- The two stuck records — the unfolded metadata win and the
  reuse-distance hit-rate signal — both finally yielded, and notably with
  *no* paying axis: tightening the layout repaid itself in locality, and
  gentler ghost admission sped the hot path by avoiding protected churn.
  Pure-representation and policy-gating changes can be genuinely free when
  they reduce work rather than add a branch.
- Hit-rate has two distinct levers on this LIRS trace: *retention* (how
  much working set protected holds) and *admission* (what earns a
  protected slot). v002's gated admission gained hit-rate for free, while
  v003's blunt 90/10 cap widening gained the same axis but starved the
  protected-MRU fast path — confirming once more that moving the
  segment geometry trades hot-path latency, whereas smarter admission does
  not.
- The metadata fold that died on infra in Cycle 0003 was sound on merit:
  packing the flag into `prev` (not a separate `bool`) is exactly what
  turns a 32-byte result into a 24-byte one, and centralized cast helpers
  avoided the per-access width tax that sank earlier attempts.

## Open questions

- v001 and v002 are independent wins on different axes (metadata vs.
  hit-rate) — can the 24-byte node and the gated ghost admission be folded
  onto one base, the way SLRU and the fast hasher stacked in Cycle 0002?
- Is there more retention headroom from admission cleverness (e.g. tuning
  the repeated-hit threshold) now that blunt cap-widening is shown to cost
  latency?

## Continuation

Promote the v001 24-byte node and the v002 gated-ghost admission as the
new bases, and next cycle attempt to combine them into a single Variant
holding both the smaller node and the new hit-rate record — they touch
disjoint structures (node layout vs. external ghost store), so the fold
should be clean. Keep pursuing hit-rate through admission gating, not
through raw protected-cap widening; drop the blunt 80→90% split as
tried-and-regressed. The Cycle 0002 SLRU base is now superseded on both
the metadata and hit-rate axes.


### Cycle 0005

# Cycle 0005 Insight

## Cohort outcome

This cycle attempted to cash the Cycle 0004 follow-ups: fold the gated
ghost admission onto the 24-byte node (v001), tighten admission with a
two-hit probation gate (v002), and fuse the promote+demote pair to cut
hot-path churn (v003). All three held the 24-byte node class and all
three landed net neutral-to-negative on their bases — none advanced the
frontier. The standing champion (Cycle 0004's 24-byte node) was not
displaced.

## Patterns

- Every "free mechanism" claim held but every policy it encoded moved
  the wrong way. Spare-high-bit flags (`REF_MASK`/`PROT_MASK`), the
  in-struct ghost store, and pure control-flow refactors each added zero
  node bytes — yet the gated ghost fold, the two-hit gate, and the fusion
  all regressed latency and/or hit-rate. The size mechanism is proven
  cheap; the behavioral lever is what failed.
- Workload-a's reuse is short-distance, and that drains the admission
  levers. Ghost admission fired too rarely to move recall; the second-hit
  gate delayed promotion of genuinely-hot keys and pushed hit-rate below
  parent. The hit-rate headroom that admission cleverness opened in
  Cycle 0004 did not reproduce here — it looks trace/size-contingent, not
  a general win.
- "Fusion" of two correct list-surgery calls is not redundancy removal:
  the demote-on-overflow path already does exactly four surgeries, so
  collapsing the calls performs the same work plus a hot-path branch and
  a parallel code path — regressing even the untouched `put`, the tell of
  codegen perturbation rather than algorithmic saving.

## Open questions

- Is the Cycle 0004 gated-ghost hit-rate gain actually realizable on
  workload-a, or was it specific to the trace/size it was measured on?
  v001's near-zero recall delta is a direct warning that the fold's
  premise may not hold here.
- With admission gating now tapped out on this workload, where does
  remaining headroom live — back on the latency/layout axis, or is the
  current champion close to this trace's ceiling?

## Continuation

Drop the v001 ghost-onto-24-byte fold: it is structurally per-op-costly
while its benefit is absent on workload-a — the cross-cycle additive
prediction did not hold. Drop two-hit admission gating and the
promote/demote fusion as tried-and-regressed. Stop chasing hit-rate via
admission cleverness on this workload; before any further policy work,
the next cycle should verify whether the Cycle 0004 hit-rate record even
reproduces on workload-a, and otherwise pivot back to latency/layout
levers. Cycle 0004's 24-byte node remains the standing base.


### Cycle 0006

# Cycle 0006 Insight

## Cohort outcome

This cycle ran two latency micro-optimizations against the 24-byte champion
and one geometry-sizing probe. Both micro-opts — eliding arena bounds
checks via unchecked indexing (v001) and fusing the unlink+push_back
relink to drop "dead" sentinel writes (v002) — held hit-rate and node size
but regressed per-op latency, advancing nothing. The lone advance was v003:
nudging the protected soft cap from 4/5 to 7/8 of capacity recovered
hit-rate above the parent with no latency penalty.

## Patterns

- Both latency micro-opts failed for the same reason and it is a cost-model
  error: the "redundant work" each removed was already free. Bounds checks
  on sequentially-predicted, proven-in-bounds arena accesses were elided or
  cheap (v001); the eliminated NIL stores were store-buffer-absorbed writes
  to an already-resident cache line (v002). In both cases the change traded
  a non-cost for real codegen damage — unsafe risk and register/code-size
  pressure from inlining — and came out net slower. The hot-path latency of
  this SLRU lives in hashing, link surgery, and cache-line traversal, not in
  the checks or writes these variants targeted.
- The Cycle 0004 lesson that protected-cap widening costs latency is now
  shown to be magnitude-dependent, not categorical. v003's modest one-eighth
  step bought hit-rate for free — get/put p50 even ticked down — whereas
  v003-from-Cycle-0004's larger share starved the fast path. There is real
  headroom between the parent's 4/5 geometry and the point where latency
  breaks down.

## Open questions

- How far can the protected fraction be pushed past 7/8 before the
  fast-path starvation seen at larger shares reappears — is there a clean
  cap maximizing hit-rate while latency holds?
- With the obvious micro-opt levers tapped, is the 27 ns champion latency
  near this trace's floor, or does a structural change (hashing, traversal
  layout) still have room the redundant-work edits never reached?

## Continuation

Promote v003's 7/8 protected cap as the new base — it is a free hit-rate
gain and the only frontier mover this cycle. Next cycle, probe the
remaining geometry headroom (a fraction between 7/8 and the starvation
point) rather than reaching straight for the largest share. Drop
bounds-check elision and the relink fusion as tried-and-regressed: both
attacked non-bottlenecks. If pursuing latency further, target the actual
hot-path costs (hashing, link surgery, cache-line traversal), not
compiler-handled redundancy.


## Rejected variants

- v003 — fitness
  caffeine simulator failed for cycle/0007/v003: caffeine simulator failed (exit 1, trace=lirs:inputs/traces/workload-a.trace.gz):
  	at java.base/java.util.stream.ReferencePipeline$7$1FlatMap.accept(ReferencePipeline.java:288)
  	at java.base/java.util.stream.ReferencePipeline$2$1.accept(ReferencePipeline.java:197)
  	at java.base/java.util.stream.ReferencePipeline$3$1.accept(ReferencePipeline.java:214)
  	at java.base/java.util.Iterator.forEachRemaining(Iterator.java:133)
  	at java.base/java.util.Spliterators$IteratorSpliterator.forEachRemaining(Spliterators.java:1939)
  	at java.base/java.util.stream.AbstractPipeline.copyInto(AbstractPipeline.java:570)
  	at java.base/java.util.stream.AbstractPipeline.wrapAndCopyInto(AbstractPipeline.java:560)
  	at java.base/java.util.stream.ReduceOps$ReduceOp.evaluateSequential(ReduceOps.java:921)
  	at java.base/java.util.stream.AbstractPipeline.evaluate(AbstractPipeline.java:265)
  	at java.base/java.util.stream.ReferencePipeline.collect(ReferencePipeline.java:723)
  	at com.github.benmanes.caffeine.cache.simulator.policy.Registry.policies(Registry.java:117)
  	at com.github.benmanes.caffeine.cache.simulator.Simulator.getPolicyActors(Simulator.java:122)
  	at com.github.benmanes.caffeine.cache.simulator.Simulator.run(Simulator.java:71)
  	at com.github.benmanes.caffeine.cache.simulator.Simulate.simulate(Simulate.java:107)
  	at com.github.benmanes.caffeine.cache.simulator.Simulate.lambda$run$0(Simulate.java:80)
  	at com.google.common.collect.Maps.toMap(Maps.java:1300)
  	at com.google.common.collect.Maps.toMap(Maps.java:1278)
  	at com.github.benmanes.caffeine.cache.simulator.Simulate.run(Simulate.java:78)
  	at picocli.CommandLine.executeUserObject(CommandLine.java:2045)
  	at picocli.CommandLine.access$1500(CommandLine.java:148)
  	at picocli.CommandLine$RunLast.executeUserObjectOfLastSubcommandWithSameParent(CommandLine.java:2469)
  	at picocli.CommandLine$RunLast.handle(CommandLine.java:2461)
  	at picocli.CommandLine$RunLast.handle(CommandLine.java:2423)
  	at picocli.CommandLine$AbstractParseResultHandler.execute(CommandLine.java:2277)
  	at picocli.CommandLine$RunLast.execute(CommandLine.java:2425)
  	at picocli.CommandLine.execute(CommandLine.java:2174)
  	at com.github.benmanes.caffeine.cache.simulator.Simulate.main(Simulate.java:133)
  
  FAILURE: Build failed with an exception.
  
  * What went wrong:
  Execution failed for task ':simulator:simulate'.
  > Process 'command '/home/yaniv/.gradle/jdks/azul_systems__inc_-25-amd64-linux.2/bin/java'' finished with non-zero exit value 1
  
  * Try:
  > Run with --stacktrace option to get the stack trace.
  > Run with --info or --debug option to get more log output.
  > Get more help at https://help.gradle.org.
  
  BUILD FAILED in 2s

## Write

Exactly one file: `cycle-insight.md` in the output dir.
