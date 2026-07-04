# Variant v003 Insight

## Hurt

- **hit-rate**: Eliminating promotion logic and tier distinction removes the adaptive feedback mechanism that identified frequently re-accessed items; without the ability to promote cold-stash hits to a hot tier, the cache loses the signal of which evicted items are worth keeping warm, forcing all stash lookups through linear search regardless of access frequency.

## Takeaway

The two-tier structure with promotion-on-access was not merely organizational complexity—it was a working feedback mechanism. By consolidating to a single unified stash and removing promotion logic, the variant sacrifices the ability to learn which evicted items deserve warmer placement or re-insertion into the primary buckets. The asymmetric 18%/7% partition in the parent was optimized to protect frequently re-accessed items; the uniform 25% stash has no such optimization, treating all stashed items identically and leaving them subject to O(n) linear search regardless of how often they are accessed.
