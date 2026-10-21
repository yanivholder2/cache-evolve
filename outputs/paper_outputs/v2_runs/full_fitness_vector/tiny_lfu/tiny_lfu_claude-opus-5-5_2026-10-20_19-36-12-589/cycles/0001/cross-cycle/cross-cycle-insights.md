## Recurring wins

- Only one cycle so far, so no pattern has held across cycles yet. The single confirmed win is tentative.
- Making each operation cheaper works. Replacing the repeated per-row SipHash calls with a fast hash and a tighter sketch layout lowered get and put latency at both p50 and p99. It also shrank memory a little and did not hurt hit rate. The put tail gained most. v003 is now the baseline.

## Recurring failures

- Adding resident-list structure (W-TinyLFU window + SLRU) without a fresher frequency signal cost memory and latency and gave almost no hit-rate gain. Region tags, extra arrays and relinking on every hit are pure overhead unless hit rate rises with them.
- Infra failures can block a whole hypothesis. An HTTP 429 from Maven while downloading the Caffeine simulator stopped the sketch-aging variant before it ran. That rejection says nothing about the idea itself.

## Open questions

- Sketch aging/reset with a lower saturation cap is untested. Does it raise hit rate on workload-a? The current evidence suggests the stale admission signal, not the list structure, is what caps hit rate.
- Once the frequency signal stays fresh through aging, do window/SLRU structures start to pay off? Does window size above about 1% matter?
- Is the unkeyed fast hasher acceptable if a trace has adversarial or collision-prone keys?

## Per-policy notes

- TinyLFU baseline (CountMinSketch admission): per-op cost came mostly from hashing, not from the cache logic. Hit rate stayed flat across every variant that kept the original counting rules.
- W-TinyLFU (window + SLRU): this is a structural change with a real cost. Revisit it only after aging has been shown to move hit rate, and keep the relink cost on each hit low.
- TinyLFU + sketch aging: not yet evaluated. Re-submit it on top of the fast-hash baseline.
