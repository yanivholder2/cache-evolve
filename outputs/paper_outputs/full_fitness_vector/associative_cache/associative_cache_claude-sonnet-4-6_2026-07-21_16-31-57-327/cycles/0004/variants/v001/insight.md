# Variant v001 Insight

## Helped

- **hit-rate**: Replacing the single conditional swap in `ClassicLRU::get_from_position` with a loop capped at 3 swaps lets warm mid-bucket items bubble to the head in fewer accesses, reducing their exposure to tail eviction before they can accumulate enough hits to dominate.

## Hurt

## Takeaway

Extending the per-access promotion budget from one swap to three delivers a measurable hit-rate gain on the Zipfian trace without any per-op-time cost, because the early-break on a failed swap means items that are already near the front (the common case under Zipfian skew) exit the loop after a single iteration. The result confirms that v003's single-swap cap was the bottleneck: items at distance 2–3 from the head were being evicted before they could reach the protected zone, and three swaps is sufficient to close that gap for the dominant hot-key population.
