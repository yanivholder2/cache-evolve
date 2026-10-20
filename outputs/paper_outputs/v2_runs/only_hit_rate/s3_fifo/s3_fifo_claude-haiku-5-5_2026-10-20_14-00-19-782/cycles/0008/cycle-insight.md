# Cycle 0008 Insight

## Cohort outcome

The cohort tried a cheaper hasher on the resident index (v001), moving small-queue second-hit entries to the front of main earlier (v002), and a counting pre-filter in front of the ghost linear scan (v003). All three Winners are hit-rate neutral against the parent, so none is a demonstrated gain; the cohort is cost-side work that has not yet been measured on cost.

## Patterns

- All three Winners held hit-rate flat, which fits their intent: each changes how work is done or when a warm key moves, not which keys are evicted. None reported per-op-time or metadata-size, so every cost claim remains unmeasured.
- Two of the three edits (v001 hasher, v003 pre-filter) are constant-factor or metadata changes on the miss path. The ghost linear scan is still untouched by v001, so any gain from it is bounded.
- The promotion-timing change (v002) showed that moving an already-warm key earlier does not alter residency. Promotion-timing levers need to change the eviction decision itself to matter.
- v003's implementation diverges from its hypothesis: counters are wider than specified, and the key is hashed on every ghost push, pop and re-admission, so the eviction path may slow even if skipped scans save time. The cost direction is genuinely uncertain.
- Overlay edits were not confirmed for any variant this cycle, the same verification gap carried since earlier cycles.
- Evidence is still one trace (lirs workload-a) at one capacity (2048).

## Open questions

- Does the hasher swap (v001) lower per-op-time on get and put, and does the ghost scan cap that gain?
- Does the ghost pre-filter (v003) save more on skipped scans than it costs in per-push hashing and wider counters? Only a rerun reporting per-op-time and metadata-size can answer this.
- Did the v001 and v003 edits land as described in the built code, and did v002's promotion edit land at all?

## Continuation

Carry the hasher swap (v001) and the ghost pre-filter (v003) forward only as cost candidates, and require a rerun that reports per-op-time and metadata-size plus confirmation of each overlay edit before calling either a win; test v003 with the counter width the hypothesis specified. Drop the earlier promotion of second-hit small entries (v002) as a standalone change, since it moved timing without changing hits. Do not spend the next cycle on further promotion-timing tweaks; prioritize build and overlay verification and cost reporting.
