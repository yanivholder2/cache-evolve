# Variant v001 Insight

## Helped

- hit-rate: Widening `prot_cap` from `capacity * 7 / 8` to `capacity * 15 / 16` in `try_new` keeps more promoted entries resident before demotion, nudging hit-rate above the parent as the geometry probe predicted.

## Hurt

- per-op-time: The wider protected segment starves the protected-MRU fast path — get and put p50 and p99 all jumped well above the parent, the exact latency regression the hypothesis warned would signal returning starvation.

## Takeaway

Stepping the protected soft cap to 15/16 confirms there is a starvation cliff between 7/8 and 15/16: the extra hit-rate is real but small, while latency degrades sharply because the near-full protected segment leaves the fast path little room to operate. metadata-size held at 24 bytes since only the sizing fraction changed, but the trade is unfavorable — 7/8 sits below the latency breakdown that 15/16 crosses.
