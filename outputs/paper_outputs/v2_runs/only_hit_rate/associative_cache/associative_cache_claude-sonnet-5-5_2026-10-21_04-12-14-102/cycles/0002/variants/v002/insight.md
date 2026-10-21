# Variant v002 Insight

## Helped

- (none): hit-rate, the only dimension measured, did not move relative to the parent. Frequency-aware victim choice among the 4 oldest entries made no measurable difference.

## Hurt

- (none measured): Only hit-rate was evaluated, so the extra cost of the per-entry counter is untested. The counter adds a u8 per entry (metadata-size), and the front-K scan plus the halving on every full-bucket insert should add work to the miss path (per-op-time). Neither dimension was recorded, so there is no evidence either way. I could not list or read the overlay source in this session, so this comparison rests on the hypothesis text and the fitness vectors.

## Takeaway

Hit-rate came out identical to plain LRU in the parent, which suggests the frequency-aware window (K=4 of 64 ways, with halving aging) rarely picked a different victim than the oldest entry, or its choices were neutral on this LIRS trace. The scan-pollution protection the hypothesis expected did not appear at this window size. Any real effect would need a wider candidate window, or the change may not have taken effect. With no cost dimensions measured, the added complexity has no demonstrated payoff.
