# Variant v002 Insight

## Helped

- (none): The settling-victim rule moved no dimension upward — hit-rate did not rise over v003's blind-LRU cascade despite the extra `pick_settling_victim` scan that prefers displacing residents with a clean second home.

## Hurt

- hit-rate: Effectively flat, a hair below the parent — swapping the cascade's victim from the LRU to an occupancy-aware "settling" resident changed *which* key gets carried onward but not *how many* keys end the cascade seated, so the terminal-drop count the hypothesis targeted did not fall.

## Takeaway

On this workload and stash size, victim *quality* inside the cuckoo cascade is not an independent lever from cascade *depth*: full-alternate hops are rare enough, and a displaced settling victim's own clean home is reachable in so few cases, that choosing a tidier resident to evict produces the same survivor set as displacing the LRU. The depth that v003 added captured the available slack; refining who gets bumped at each hop adds scan cost without converting more drops into seats.
