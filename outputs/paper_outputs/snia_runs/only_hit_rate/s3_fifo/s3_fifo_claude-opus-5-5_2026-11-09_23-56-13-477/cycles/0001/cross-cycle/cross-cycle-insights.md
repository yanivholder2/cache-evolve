# Cross-Cycle Insights

The ledger holds 1 completed cycle (0001). Every item below comes from that one cycle, so treat it as tentative until a second cycle confirms it.

## Recurring wins

- None confirmed yet. One cycle is not enough to call anything recurring.
- Candidate: changing the data structures without touching the policy (hash index, slab) kept hit-rate identical to the baseline. That makes it a safe parent for later work if it turns out to be faster.
- Candidate: loosening the graduation threshold gave the best result, but only by a hair. It is weak evidence and has not been reproduced.

## Recurring failures

- Changing one constant at a time didn't clearly help, whether the constant was the graduation threshold or the small/main split. Each change gained on hm_0 and lost on web_0, so aggregate hit-rate barely moved.
- Changing what happens to an entry after it reaches main didn't help. Resetting a graduate's freq didn't undo the pollution caused by admitting it, so the cost seems to be paid at the moment a key is admitted to main.

## Open questions

- Are per-op-time and metadata-size actually being measured? Does the index/slab refactor speed things up, and does its extra state make metadata-size worse?
- Does a small queue somewhere between 10% and 25%, combined with a graduation threshold of 1, keep the hm_0 gain without the web_0 loss?
- Is there an online signal that tells hm_0-like traffic apart from web_0-like traffic, such as how much spare room main has or how often keys are reused in the small queue? If so, graduation into main could adapt to it.

## Per-policy notes

- S3-FIFO: the two policy knobs work together, not separately. The graduation threshold limits how well a smaller small queue can do, and unfiltered admission limits how well a looser threshold can do. Change both together, or make admission depend on main having room or on the observed reuse rate.
- S3-FIFO: the traces pull in opposite directions. hm_0 rewards easy entry into main and more main capacity. web_0 penalises both, because keys accessed exactly twice pollute main and medium-reuse keys get ghosted too early.
