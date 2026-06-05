# Variant v002 Insight

## Hurt

- hit-rate: Eliminating the target-tier retry for foreign residents removes their "one-more-chance" to prove themselves in their true target bucket before demotion to stash, preventing items that could have recovered from doing so and reducing overall hit rate.

## Takeaway

Bypassing the target tier for foreign residents evicted from probing trades off recovery opportunities for simplicity. Items that land in probing as foreign residents (due to hash collision placement or lack of target space) previously had a second chance to demonstrate value in their true target before final demotion to stash. Eliminating this retry path drops them directly to stash, a lower-priority tier with fewer access opportunities, cutting hit-rate by ~3%. The one-more-chance principle appears fundamental to the cascade design: each demotion step acts as a safety valve, and removing it at the foreign-resident boundary degrades the system's ability to retain borderline-useful items, counteracting the intended simplification goal.
