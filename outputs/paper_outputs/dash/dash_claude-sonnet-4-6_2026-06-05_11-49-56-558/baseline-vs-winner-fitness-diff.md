# Baseline vs winner — three-dimensional fitness

Per-dimension summary score and total weighted score (config weights:
hit-rate 0.6, metadata-size 0.2, per-op-time 0.2). Units: hit-rate = mean hit
rate (higher better); metadata-size = max per-instance bytes (lower better);
per-op-time = get p50 ns (lower better). Weighted score is signed percentage
change vs baseline, so baseline = 0 and only a positive score net-beats it.

| Variant         | hit-rate | metadata-size | per-op-time | Weighted score |
| --------------- | -------- | ------------- | ----------- | -------------- |
| baseline        | 0.6634   | 40            | 118         | 0.0000         |
| cycle/0009/v003 | 0.7240   | 40            | 145         | +0.0090        |

Winners net-beating the baseline (positive score): cycle/0009/v003. hit-rate is from the run's records; metadata-size and per-op-time
were re-measured together on one machine (per-op-time is wall-clock,
machine-dependent).
