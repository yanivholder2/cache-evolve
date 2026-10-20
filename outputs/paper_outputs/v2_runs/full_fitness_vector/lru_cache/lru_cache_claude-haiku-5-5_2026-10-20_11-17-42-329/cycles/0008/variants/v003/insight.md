# Variant v003 Insight

Note: the overlay source and the eval/ directory could not be inspected in this
session (no shell or search tool was available, and the file paths I tried did
not exist). Code-level claims below are inferred from H03 and from the two
fitness vectors, and should be checked against the overlay diff.

## Helped

_No dimension improved beyond measurement noise._

## Hurt

- hit-rate: Admitting a ghost-ring hit at the probationary head with REFERENCED clear means a returning key must earn protection with a second hit before it is promoted. On the scan-heavy workload-a trace, that second hit apparently comes too late, so the variant gives back the protected-direct admission the parent used and loses hit rate relative to the parent.
- per-op-time: The put tail latency rose sharply, which is more than a single list-link choice on the miss path would normally cost. Either other edits from the hypothesis landed in the overlay, or the put-path measurement is noisy; the diff needs checking before attributing this to the admission change.
- metadata-size: The LruCache struct grew by one word, which matches the ghost-hit counter H03 asked for. The ring's heap storage is still not counted, so the real memory footprint of the ghost mechanism remains unmeasured.

## Takeaway

On workload-a at 2048, sending ghost-ring hits straight to protected outperforms sending them to probation, so H03's premise that probationary admission would recover hit rate does not hold for this trace. The get path is unchanged within noise, so the regression is a policy effect rather than a hit-path cost. Keep the parent's protected-direct admission and treat the put-tail regression as an open question to resolve from the overlay diff before any further admission-policy work.
