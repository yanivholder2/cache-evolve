# Variant v002 Insight

## Helped

_(none — hit-rate was unchanged)_

## Hurt

_(none — hit-rate was unchanged)_

## Takeaway

Replacing strict LRU demotion from protected with a 4-node frequency scan (freq capped at 3, reset on every demotion) produced no measurable change on workload-a. The scan window covers only a tiny fraction of the 2048-slot protected segment, so the "best" target it finds is almost always adjacent to `prot_head` — the same node strict LRU would have chosen. Additionally, the cap-at-3 / reset-on-demotion design means the freq counter cannot accumulate durable signal across demotion cycles: a genuinely high-reuse key that gets demoted re-enters protected with freq=0, indistinguishable from a newly promoted one-hit entry the next time an overflow scan runs.
