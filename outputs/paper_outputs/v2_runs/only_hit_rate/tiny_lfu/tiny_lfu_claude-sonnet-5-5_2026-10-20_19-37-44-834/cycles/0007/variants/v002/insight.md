# Variant v002 Insight

## Helped

- None. No dimension improved. I could not read the overlay source in this session, so this rests on the hypothesis text and the fitness vectors.

## Hurt

- hit-rate: It dropped slightly against the parent. With the extra `put` increment removed from the new-key path, a first-time missed key reaches frequency 1 instead of 2. The tie-admit rule `cand_freq >= 2` then rejects candidates that previously got in on their first miss. Some of those candidates were worth admitting, so the double count was helping more than it hurt.

## Takeaway

The double count on a get-miss followed by a put is not a bug on workload-a at size 2048. It acts as a cheap, useful admission bias toward newly seen keys, and removing it makes the unchanged `>= 2` threshold stricter. The effect is small, so the sketch's count accuracy is not the main limiter. If a cleaner count is wanted, the threshold would need to be retuned (for example to `>= 1`) at the same time. Changing only the counting does not work.
