# Baseline Insight

S3-FIFO cache: a small FIFO admission queue, a main FIFO resident queue, and a
ghost queue of recently evicted keys. Reused small-queue entries graduate into
the main queue, cold ones leave their key in the ghost queue, and a ghost hit
re-admits straight into the main queue. Main-queue eviction is a second-chance
pass over a saturating frequency counter. The only configurable knob is
`capacity`; the small/main/ghost partition is derived from it.

This is the seed baseline: the starting point every cycle-1 cohort is proposing
improvements over. Measured fitness lives in `eval/fitness-vector.json`.
