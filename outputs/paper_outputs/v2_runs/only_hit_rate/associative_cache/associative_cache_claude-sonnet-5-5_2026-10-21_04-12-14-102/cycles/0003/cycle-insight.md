# Cycle 0003 Insight

## Cohort outcome

Three refinements of the probation/protected split were tried: a smaller probation segment (v001), a per-bucket ghost ring that promotes ghost hits straight to protected (v002), and an adaptive 2Q-style probation target with ghost history (v003). Only the smaller probation segment won, with a small hit-rate gain. The ghost-promotion variant did not improve on its parent.

## Patterns

- Tuning the split size gave only a small gain, so most of the benefit comes from having the split at all. This matches cycle 0002.
- Ghost-history ideas did not add value on top of the split. A ghost hit looks like weak reuse evidence on this trace and promoting on it can displace entries that had earned protection.
- v003's insight compares against plain LRU and credits the whole split, not an increment over the current parent. It is hard to read as a new gain, and it did not win.
- Per-op-time and metadata-size were unmeasured for the third cycle running. Every added piece of state (ghost rings, adaptive targets, markers) is still unpriced.

## Open questions

- Where does the smaller-probation trend turn? Is a smaller share still better, or is the optimum near the current one?
- Does a fixed probation size match an adaptive one, making the extra ghost state unnecessary?
- Would a stricter promotion rule (ghost hit plus a second signal) rescue ghost history?
- Do the gains survive once speed and metadata cost are measured?

## Continuation

Build on v001's smaller probation size and probe a few more sizes to find the turning point. Drop ghost-hit direct promotion and further ghost-ring variants unless promotion is made stricter. Fixing the evaluation so per-op-time and metadata-size appear in the fitness vector should come before adding any more bookkeeping.
