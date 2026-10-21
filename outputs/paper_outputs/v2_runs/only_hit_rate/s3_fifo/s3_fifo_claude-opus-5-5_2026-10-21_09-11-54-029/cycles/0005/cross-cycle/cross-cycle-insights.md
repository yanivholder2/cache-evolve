## Recurring wins

- Workload-a consistently rewards protecting keys with proven reuse. Every win across five cycles either gave main more capacity or more retention, or widened the ghost's evidence without making entry easier.
- Partition shrinking paid off twice (default→10% in cycle 0001, →5% in cycle 0002), with diminishing returns. The gain comes from a bigger main queue, not from a bigger ghost.
- Main-queue retention through the frequency cap gave a real gain from 3 to 7 (cycle 0003), then flattened (7→15 in cycle 0004 was within noise).
- Ghosting main-queue eviction victims (cycle 0005) gave a small gain by widening the reuse evidence while leaving re-admission at front-of-main with freq 0.
- Behaviour-neutral structural ports (indexed slab, index and lazy ghost) were twice verified as exact against the linear reference (cycles 0003 and 0004). Policy and representation can be changed independently.
- Single-lever variants gave clean attributions. The one bundled change (cycle 0001 v002) could not be attributed.

## Recurring failures

- Making ghost re-admission more generous lost every time: a freq-1 credit (cycle 0002) and a doubled or decoupled ghost (cycle 0002). One ghost hit is enough evidence of reuse, and extra credit takes main capacity away from keys with proven reuse.
- Making ghost re-admission stricter also lost every time: tail insertion (cycle 0003) and small-queue probation with a banked credit (cycle 0004). The small queue's window is far shorter than the ghost's reuse horizon, so probation rejects the keys the ghost correctly identified. Front-of-main at freq 0 is settled.
- Changing graduation from small to main was negative or flat. Earlier graduation with a frequency reset regressed (cycle 0001), and a 1-hit threshold alone was flat (cycle 0005), because the ghost already recovers keys hit once in small.
- Raising the frequency cap above 7 and aging credit through decay or halving (cycles 0004 and 0005) were flat or negative. Linear decrement at cap 7 looks near-optimal, and faster aging hurts keys with moderate reuse.
- Evaluation coverage is the most persistent process failure. Per-op-time and metadata-size have gone unreported for all five cycles even though every Continuation asked for them. Only one trace at one cache size has ever been tested.

## Open questions

- What are the per-op-time savings and metadata-size costs of the indexed structure (duplicated keys, stale ghost deque, compaction spikes)?
- Does the cumulative extra work in the incumbent lineage carry a hidden per-op-time cost? That work comes from a larger ghost, longer sweeps under higher caps, and ghost inserts on main evictions.
- Do the accumulated small wins (5% split, cap 7, ghosting main victims) generalize to other traces and cache sizes, or are they overfitted to workload-a?
- Would a separate or split ghost budget for main victims grow the cycle 0005 gain, or would it only show how the shared budget dilutes small-queue rejects?
- Is any hit-rate headroom left on workload-a at this size? Effects have been near noise since cycle 0004, which suggests the trace is close to saturated for this policy family.

## Per-policy notes

- S3-FIFO (the only policy explored so far): the current baseline is the indexed structure with a 5% small queue, main cap 7 with linear decrement, re-admission at front-of-main with freq 0, and main victims recorded in the shared ghost.
- S3-FIFO levers settled as flat or negative: re-admission credit, tail insertion and probation; ghost enlargement alone; graduation threshold; cap above 7; credit decay; shrinking the partition below 5%.
- S3-FIFO levers still open: a ghost budget for main victims, plus all cost-side and generalization measurements.
