# Variant v002 Insight

## Helped

- hit-rate (`web_0`, 16K and 128K): Both cells rose slightly, in the direction the hypothesis predicted. Segments where far-credited promotees were demoted unhit drove `far_score` negative, and from then on far ghost hits entered the stash as fresh keys. That removed some of the stale far credit that c7 found hurts `web_0`. The gain is tiny, though, which means far credit was a small part of `web_0`'s ghost traffic, or that the gate was rarely closed for long. Three things keep the gate open by default or reopen it: the score starts at 0, `>= 0` counts as "on", and the 1-in-8 probe keys keep earning rewards.

## Hurt

- hit-rate (`hm_0`, 128K): This is the cell the hypothesis meant to protect, and it slipped slightly. So the feedback signal does not fully separate useful far credit from useless far credit here:
  - The score rewards only a *resident* re-hit during the item's first normal-bucket stay. It penalizes every unhit exit through `demote_from_normal_bucket`, including the foreign-retry cascade and CLOCK victims.
  - In a saturated 2-slot bucket, many far-credited promotees on `hm_0` are pushed out before their next reference. Those are penalized even when the key would have come back later, so some segments shut off far credit that was paying off.
  - Once a far hit goes uncredited, its fingerprint is still consumed. The key loses its ghost signal for that return, and only the 1-in-8 probe keys can pull the score back up.
- hit-rate (`hm_0`, 16K): No change. Under heavy saturation the near/far split and the score rarely change which key gets promoted, which matches the parent's finding that the ghost-carried bit is inert at this size.

## Takeaway

A per-segment ±1 score of "re-hit while resident vs. evicted unhit" does move far credit in the right direction for `web_0`, but it is too blunt to protect `hm_0` 128K. On that trace, far-credited keys pay off on later returns more than on immediate resident re-hits, so the score counts them as failures. If adaptive far credit is pursued further, the reward has to count re-references over a longer horizon, for example a later ghost hit on a key that had been far-credited. The current one-stay resident check is not enough. Short of that, this mechanism nets out to roughly the parent's fixed factor-2 behaviour.
