# Cycle 0010 Insight

## Cohort outcome

The cohort tried a capped side ring for main-evicted zero-frequency keys (v001), replacing the linear ghost queue with a deque plus key index (v002), and feeding main-queue victims with frequency of at least 1 into the ghost queue (v003). Only v002 is a Winner, and it is hit-rate neutral, so its case rests on an unmeasured cost claim. v001 lost hit-rate and v003 was flat.

## Patterns

- The side-ring split (v001) lost the hit-rate gain that the Cycle 0009 main-eviction ghost feed produced. Keeping main-fed entries in the shared ghost window, with no tight cap, appears necessary for that gain.
- v003 fed warm victims (frequency of at least 1) into the ghost queue and was flat. Together with v001, this points to the Cycle 0009 gain coming from the cold, zero-frequency victims. Warm-key feeding adds nothing on this trace.
- v002 is a ghost-structure refactor meant to cut miss-path cost with eviction unchanged. Hit-rate held flat, as predicted, but no per-op-time or metadata-size was reported, so the speed claim is unmeasured.
- The v002 and v003 insights could not confirm their overlay edits in the built code. A flat result from v003 may be a no-op implementation, so the gap from prior cycles still applies.
- Evidence is still one trace (lirs workload-a) at one capacity (2048).

## Open questions

- Is the Cycle 0009 gain specific to cold victims, and does a cold-only feed hold on other traces and capacities?
- Does the v002 deque-and-index refactor lower per-op-time on the miss path, and at what metadata cost?
- Did the v002 and v003 edits land in the built code, and did v003's ghost push fire at all?

## Continuation

Carry the cold-victim main-eviction ghost feed from Cycle 0009 (v002 of that cycle) forward as the lead hit-rate candidate. Do not pursue a capped side ring for main-fed entries (v001) or a warm-victim feed (v003) as standalone changes. Treat the ghost refactor (v002) as a cost candidate only, and require a rerun that reports per-op-time and metadata-size plus overlay confirmation before calling it a win.
