# Variant v002 Insight

> Note: I could not open the overlay source in this session. I had no directory listing or search tools, and `manifest.toml` was not at the expected paths. The mechanism below comes from H02's stated change: add a "hit while in normal tier" flag, and send untouched demoted normal-tier victims to stash LRU instead of MRU, keeping their pre-credit. I did not check the diff itself, including whether the flag reused an existing counter or reference bit.

## Helped

## Hurt

- hit-rate: Sending untouched demoted normal-tier victims to the stash LRU (probation) end dropped hit-rate sharply against the parent. This is a large loss, not noise. Many keys that went unhit during their normal-tier stay are re-referenced soon after demotion. At probation they are evicted from the stash before that reuse arrives, so the pre-credit, which only matters on a stash hit, never pays off.

## Takeaway

Demotion-to-MRU was not an unfiltered leak. It acted as a deliberate second chance. A key that earned promotion into the normal tier carries reuse evidence even if the normal tier evicts it before its next access. On this LIRS trace, that evidence beats a fresh key's, and protecting it at stash MRU is worth the slots it takes from ghost-gated and hot keys. The pattern from 0001–0009, where each tighter filter on protected stash space helped, does not reach this route. The hypothesis's own falsification branch holds: demotion placement looks to be at or near its optimum. An untouched normal-tier stay is a poor signal for "no reuse" here, likely because the normal-tier residency window is shorter than these keys' reuse distance.

One caveat: if the implementation reused an existing counter or reference bit that does not track exactly "hit since landing in the normal tier", some hot keys may have been misclassified and sent to probation. That would make the loss larger, so check the diff before ruling out the direction entirely.
