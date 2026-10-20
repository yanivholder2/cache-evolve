# Variant v001 Insight

## Helped

- per-op-time: Clearing the referenced flag and retrying demotion on the next candidate removes the unlink and push that the parent took on every referenced protected-head demotion, so both `get` and `put` get faster at p50 and p99.

## Hurt

- hit-rate: The rescued head now keeps its place in the list instead of moving to the protected MRU end, so it loses the extra rotation that protected it from demotion and gives back a small part of the hit-rate gain the parent's second chance had won.

## Takeaway

The per-op gain shows that the list surgery in the parent's demotion branch was the cost, not the second-chance idea itself. The small hit-rate loss suggests the MRU rotation was doing real work, so a flag-only retry trades that protection for speed. Caveat: I could not open the overlay source in this session (Bash, Glob, and Grep were unavailable and the file paths I tried did not exist), so this reading of the code change rests on the hypothesis text and the fitness vectors, not on the diff itself.
