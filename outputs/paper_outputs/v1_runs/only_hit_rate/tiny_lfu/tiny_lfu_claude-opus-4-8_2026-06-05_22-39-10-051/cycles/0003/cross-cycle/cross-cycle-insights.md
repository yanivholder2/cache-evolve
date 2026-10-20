## Recurring wins

- Spending the *existing* long-term frequency signal more carefully beats
  redesigning the policy. The durable gains came from a lightly
  frequency-ranked victim choice and, more so, from a more accurate counter —
  not from new mechanics.
- Estimator fidelity was the single clearest lever while it lasted: widening
  the count-min sketch (reducing cell collisions) lifted hit-rate when nothing
  in the policy logic changed.
- "Wins" increasingly mean *not regressing*. Once accuracy was bought, further
  moves on the same axis only held parity — a sign the trace's headroom is
  nearly mined out.

## Recurring failures

- Any mechanism that discards, decays, gates, or withholds long-term frequency
  signal loses: sketch aging, the W-TinyLFU admission window, doorkeepers
  (even an isolated long-clear-window one across two cycles), and deeper victim
  samples that trample recency-warm / warming-hot keys.
- Spending the signal more *aggressively* backfires — it evicts keys before
  their counts accumulate. The bare `incoming_freq > victim_freq` test already
  screens one-hit-wonders, so extra admission filtering is redundant and costly.
- Levers plateau then reverse when pushed: the victim-sample win was already at
  its sweet spot, and widening it lost. Width and conservative update proved
  redundant with each other, not additive.

## Open questions

- With both policy-design and estimator-accuracy axes exhausted, is hit-rate at
  this capacity effectively capped, or is there an untried axis?
- Is there a narrower-sketch + conservative-update tradeoff that holds accuracy
  at lower memory — i.e., can CU correct contention the wasted width currently
  pays for?
- Is the workload's stability (popularity-stable, low-churn, low tail churn)
  intrinsic, or would a churnier trace revive the parked aging/window
  mechanisms?

## Per-policy notes

- TinyLFU / count-min estimator: the productive home of every win; fidelity
  drove gains until cells stopped colliding, then flattened. Conservative
  update adds nothing on top of an already-wide sketch.
- Victim selection: light frequency ranking of a few tail candidates helps;
  deepening the sample refuted.
- Admission control (window, doorkeeper): consistently refuted — parked as
  mis-sized rather than proven wrong only for the window; the doorkeeper is
  fully dead. Recency lives in LRU position and is easily trampled by these.
