# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Effectively flat. The tiny drop is within noise, and it is not a gain. Conservative update in `CountMinSketch::increment_hashed` raises only the counters that equal the row-minimum. That change only does something when a key's minimum row is itself inflated by a collision. At 16×capacity width with 4 rows, the parent's `estimate_hashed` already reads almost every key's true count. So the strict-`>` admission contest and the 8-node `select_victim` window make nearly the same decisions as before. The few that flip cancel out.

## Takeaway

Once the sketch is 16× wider (v003), the collision overcount that conservative update would remove is already gone from the minimum row. The two fixes target the same error and do not add up. The parent's rationale suggested that never-aged, ever-growing counters still leave meaningful collision error. On workload-a they do not leave enough to change admissions, so sketch accuracy at this geometry is used up as a hit-rate lever. Conservative update is still a free accuracy gain, so keep it if future work narrows the sketch to save memory. Look for further hit-rate in recency/frequency adaptation, such as aging, a window, or the admission rule, not in better counting.
