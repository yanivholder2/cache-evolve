# Cross-Cycle Insights

## Recurring wins

- Separating "seen once" from "proven," and protecting proven keys from
  transient/scan traffic, has raised hit-rate in every cohort so far.
- The promotion gate — promoting only on a genuine second reference — is
  the load-bearing lever across cycles; winners respect it.
- Letting the recency/frequency balance self-tune from ghost-hit signal
  beat hand-set fixed structure, despite extra bookkeeping.
- Ghost memory pays off when it *feeds* the promotion decision rather
  than overriding it.

## Recurring failures

- Bypassing the promotion gate keeps backfiring: readmitting a returning
  ghost key straight into the protected segment over-promotes low-reuse
  keys, displaces hot ones, and regresses hit-rate.
- Layering more structure (ghost queue) onto a fixed-split base without
  improving the admission decision did not pay off.

## Open questions

- A frequency-sketch / TinyLFU admission gate on top of the adaptive
  winner remains untested (its variants have not produced results).
- All scoring is hit-rate only, so per-access overhead from extra lists,
  ghost maps, and sketches is invisible — would the adaptive winner's
  edge survive if cost were scored?
- Do orthogonal signals (frequency + segmentation/adaptivity) compound,
  or just add overhead?

## Per-policy notes

- Flat LRU: vulnerable to one-hit-wonder / scan pollution; the baseline
  all variants improve on. Low overhead, but no scored dimension yet
  rewards that.
- SLRU (fixed split): a solid first base, but superseded; weak as a
  standalone parent unless overhead becomes scored.
- 2Q / TinyLFU-style admission: classic angles that improve over flat
  LRU but have not won; the frequency-gate idea is still unevaluated.
- ARC (adaptive): current proven base — self-tuning target driven by
  ghost hits, respects the second-touch promotion rule, carries the most
  bookkeeping.
