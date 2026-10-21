# Variant v002 Insight

## Helped

(No measured dimension improved. H02 predicted that hit-rate would beat cycle/0008/v001 by more than the ±0.001 band, and that did not happen.)

## Hurt

- hit-rate: In `evict_small()`, a graduate's `freq` is now set to 0 right before `main_push_head`. A key that earned 2–7 small-queue hits therefore enters main exactly like a freq-0 ghost re-admission. If it gets no hit in main before the SIEVE hand reaches it, it is evicted on the first pass. Under the parent it would have used its carried credit to survive one lap. The result moved slightly down, not up, and stays inside the ±0.001 band. So repeated hits in the small queue are, at worst, mildly useful evidence for main retention on workload-a. They are not the free protection for one-burst keys that the hypothesis assumed. The change can be attributed to this one reset for three reasons. The graduation threshold (`freq > 1`), the saturating counter in `get()`, clear-on-pass `evict_main()`, ghost re-admission and the 5% split are all unchanged. And `RefS3Fifo::evict_small` applies the same `tail.freq = 0`, so the full-state comparison still pins the behaviour.

## Takeaway

Under clear-on-pass SIEVE, keeping graduation and main retention as separate signals buys nothing on workload-a. Erasing a graduate's small-queue credit is neutral to slightly harmful. This suggests that keys reaching `freq > 1` within the 5% small window are mostly real reuse, not one-burst noise, so the parent's one-lap carry-over should stay. This rules out the second open question from v001's insight, "can the graduation counter be separated from main retention?", as a source of gain. Main-side filtering of newcomers now looks exhausted for graduates. Ghost re-admissions are the only newcomers left without credit.
