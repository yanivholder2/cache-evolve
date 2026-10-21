# Variant v001 Insight

## Helped

(No measured dimension improved. The eval covers hit-rate only, and it went down.)

## Hurt

- hit-rate: The variant moves a ghost-hit re-admission from the main head to the slot just head-ward of the SIEVE hand, still with `freq = 0`. That leaves it only about one or two main evictions to get a hit before the sweep reaches it. Main evicts on every ghost hit and on every graduation into a full main, so the next eviction often reaches the re-admit straight away. Its only protection is the hand entry itself being a freq-0 victim. Ghost hits on workload-a are keys whose reuse distance is about one main-queue length, too long for the small queue to catch. A window of a few evictions is far shorter than that distance. So most re-admits are evicted and ghosted again before their next access. That turns hits the head-insertion parent would have caught into repeat ghost round-trips. The eviction slots this frees go to entries that don't use them better.

## Takeaway

Placing ghost re-admissions next to the SIEVE hand is a regression. It shrinks the re-admit's grace period from about one full sweep to about one eviction. On this lineage, a ghost hit is a real long-reuse signal, and it needs roughly a full main pass of residency to pay off. Together with the earlier finding that extra credit and behind-hand insertion also lose, this suggests head insertion at `freq = 0` is a local optimum for where re-admits go. Further re-admission work should change something other than position.
