# Variant v001 Insight

## Helped

## Hurt

## Takeaway

Increasing `SKETCH_DEPTH` from 4 to 6 produced no measurable change in hit-rate: the result was identical to the parent. With the sketch already 8× capacity wide (16 384 columns at capacity=2048), the per-row collision probability is low enough that the 4-row minimum estimator already returns accurate frequency estimates — adding two extra independent hash witnesses does not further reduce the probability that every row is simultaneously contaminated for any key on this workload. The sketch's depth is not the binding constraint; further hit-rate gains will require looking outside the sketch itself, such as the SLRU segment sizing, PROMO_THRESHOLD gating, or the k=4 victim-scan window.
