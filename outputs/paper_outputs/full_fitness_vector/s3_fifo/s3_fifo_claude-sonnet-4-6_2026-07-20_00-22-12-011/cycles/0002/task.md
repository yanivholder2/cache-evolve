# Task: cycle-insight-writer

Cycle: 0002
Output dir: C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0002

## Winners

- v001
- v003
- v002

## Variant insights

- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0002\variants\v001\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0002\variants\v002\insight.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0002\variants\v003\insight.md

## Hypotheses (this cycle)

- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0002\hypotheses\H01.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0002\hypotheses\H02.md
- C:\Users\yaniv\Projects\cache-evolve\outputs\s3_fifo_2026-07-20_00-22-12-011\cycles\0002\hypotheses\H03.md

## Prior cycles digest

### Cycle 0001

# Cycle 0001 Insight

## Cohort outcome

All three variants targeted distinct structural axes — queue-size ratio, ghost-lookup algorithmic complexity, and entry memory layout — and all three won over the baseline. The Winners share no single change but collectively demonstrate that the baseline S3-FIFO leaves headroom on multiple independent dimensions simultaneously.

## Patterns

- **Small queue was over-provisioned**: Reducing the small-queue fraction from 1/4 to 1/10 improved both hit rate and per-operation latency, indicating the original split over-invested in admission filtering at the expense of main-queue residency and scan length.
- **A dominant linear scan in `put()`**: The ghost-queue membership check was an O(n) scan that nearly halved median put latency when replaced with an O(1) hash-set oracle — the single highest-leverage algorithmic fix in the cohort.
- **Entry struct size is a meaningful lever**: Moving the frequency counter out of the entry struct achieved a 33% per-entry size reduction and delivered an unexpected latency benefit via improved cache-line utilization during the scan-heavy eviction path.
- **Fixed overhead amortizes cleanly**: Both v002 and v003 grew the `S3Fifo` control struct (HashSet header and parallel freq deque headers respectively), but that overhead is per-instance and constant, not per-entry, so it is negligible at any practical cache capacity.
- **The three improvements are orthogonal**: Each change touches a non-overlapping code path (sizing formula, ghost-check in `put()`, entry layout + eviction loops), making them safe candidates for combination.

## Open questions

- What is the optimal small-queue fraction, and does it depend on workload skew — or is there a threshold below which shrinking it further hurts cold-miss handling?
- v002 still calls `ghost.retain()` (O(n)) on a ghost hit to remove the key from the deque; the hot path for ghost hits is not yet O(1) end-to-end.
- Combining v001 + v002 + v003 may expose interactions (e.g., a smaller small queue shifts eviction pressure, changing how often the ghost set is queried); composite behavior should be validated.

## Continuation

All three strategies are worth carrying forward as a combined baseline for the next cycle — they are orthogonal and collectively represent a materially improved S3-FIFO. The next cycle should prioritize: (1) a composite variant merging all three wins, and (2) eliminating the remaining O(n) `ghost.retain()` call in v002's ghost-hit path, which is the last known linear-scan hotspot in `put()`.


## Write

Exactly one file: `cycle-insight.md` in the output dir.
