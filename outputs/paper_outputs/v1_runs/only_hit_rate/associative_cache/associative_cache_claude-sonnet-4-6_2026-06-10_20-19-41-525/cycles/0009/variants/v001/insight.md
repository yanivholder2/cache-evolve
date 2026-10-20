# Variant v001 Insight

## Helped

## Hurt

- hit-rate: Sparing upper-half items from the aggressive rate-2 decay slightly shrinks the immediate cold pool produced by each warm-lock event — upper-half items sitting at counter 2 (which uniform decay collapses to cold) now stay at counter 1 (still warm), leaving the bucket with fewer ready cold candidates on the next eviction call.

## Takeaway

Position is not a reliable proxy for "genuinely hot" under a counter cap of 4: upper-half items span the full counter range 1–4, so protecting them with a gentler decay preserves lukewarm items just as often as truly hot ones. The decisive threshold is counter value (1–2 → cold, 3–4 → warm), not bucket position, which means asymmetric position-split decay yields no benefit over the uniform rate-2 decay from v003 — and marginally reduces cold-candidate availability.
