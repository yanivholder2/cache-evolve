## Recurring wins

- Every cycle's Winner reduced noise or cost in the frequency signal. None of them changed policy shape. The winners were fast hashing (c1), sampled victims (c2), a wider sketch (c3) and get-then-put dedup (c4).
- Lifetime frequency is the right signal for workload-a. Keeping long-run counts has beaten freshness every time it was tested.
- The only large hit-rate gain in four cycles came from removing collision noise with a wider sketch. Every other win was marginal.
- Strict `>` with keep-victim-on-tie held up once it was tested directly. Keep it as the contest rule.
- Isolating one lever per variant has paid off. It showed that the early aging losses were confounded with conservative update and collision noise.

## Recurring failures

- Sketch aging/halving lost or stayed flat in every form tested: short period with conservative update (c1), long period alone (c2), and very long period on a clean sketch (c4). Freshness costs hit-rate on this trace.
- Window restructuring or adaptation didn't pay off. A 1% fixed window did nothing (c1). A naive hill-climber drifted toward an LRU-like region, which this LIRS-style trace punishes (c2).
- Tie-admission and wider victim sampling both hurt (c3). They act on estimates that never age, so the lowest estimate marks the youngest resident, not the coldest one.
- Accuracy levers that fix the same error don't add up. Conservative update on top of a wide sketch was flat (c4).
- Per-op-time and metadata-size have gone unscored in all four cycles. Speed and memory claims (fast hash, dedup, wide sketch) are all still unverified.

## Open questions

- With clean, honest estimates, can structural levers recover hit-rate? Examples are an SLRU probation/protected split, a small fixed window, or a doorkeeper. So far these were tested only on the noisy sketch.
- A first-time newcomer still ties a resident accessed once, and the newcomer loses that tie. Is there a structural fix for this case that doesn't reopen tie-admission?
- How narrow can the sketch go, perhaps narrower plus conservative update, before hit-rate drops? This matters once metadata-size is priced.
- What do the per-op-time and metadata-size costs of the current lineage actually look like? A scoring run that reports them is a prerequisite.

## Per-policy notes

- **TinyLFU sketch:** Width was the decisive lever, and dedup added a small gain. Conservative update is redundant at high width. Aging is negative for workload-a. The sketch side looks exhausted as a hit-rate lever.
- **Admission contest:** Keep strict `>` with keep-victim-on-tie. Contest-rule changes keep failing because the estimates they rely on were the real bottleneck.
- **Victim selection:** Frequency-sampled victims from the LRU head gave a marginal gain. Widening the sample hurt, so treat this lever as exhausted.
- **W-TinyLFU window / SLRU:** Results so far are neutral to negative: a fixed 1% window did nothing and an adaptive window hurt. This is the main untested lever on clean estimates and is worth one re-test.
- **Hashing:** A single fast hash with a flat sketch is neutral on policy. Its speed benefit has not been measured.
