# Baseline vs winner — three-dimensional fitness

Per-dimension summary score and total weighted score (config weights:
hit-rate 0.6, metadata-size 0.2, per-op-time 0.2). Units: hit-rate = mean hit
rate (higher better); metadata-size = max per-instance bytes (lower better);
per-op-time = get p50 ns (lower better). Weighted score is signed percentage
change vs baseline, so baseline = 0 and only a positive score net-beats it.

| Variant         | hit-rate | metadata-size | per-op-time | Weighted score |
| --------------- | -------- | ------------- | ----------- | -------------- |
| baseline        | 0.6634   | 40            | 265         | 0.0000         |
| cycle/0009/v002 | 0.7190   | 40            | 223         | +0.0820        |
| cycle/0010/v002 | 0.7190   | 40            | 231         | +0.0759        |

Winners net-beating the baseline (positive score): cycle/0009/v002, cycle/0010/v002. hit-rate is from the run's records; metadata-size and per-op-time
were re-measured together on one machine (per-op-time is wall-clock,
machine-dependent).
