# Variant v002 Insight

## Helped

- hit-rate (web_0 @ 16384): This is the cell the hypothesis targeted, and it moved up by only a trace amount. When a weak returnee gets a quick small hit, it now reaches freq 2 and graduates instead of being re-ghosted. That saves one ghost round-trip miss per such key, but few web_0 keys take that path, so the parent's web_0 loss is mostly still there.

## Hurt

- hit-rate (hm_0 @ 16384, hm_0 @ 131072, web_0 @ 131072): Each cell slipped by a hair. That fits returnees that only had a burst of hits now graduating straight into a full main and forcing an `evict_main`, which the parent's cap had pushed back to the ghost. The effect is small enough to be close to noise.

## Takeaway

Exempting weak ghost returnees from the correlated-reference cap is close to a no-op. A returnee already enters small with freq 1. The cap only stopped it at freq 1 when its only small hits came within `small_size / 8` insertions of re-entry and none came later. Even then, the cost was a single extra miss: the key was re-ghosted with freq 1, and its next return went straight to main. So the web_0 @ 16384 regression from cycle-0009 v003 comes almost entirely from **first-time** keys whose quick reuse is real, not from returnees. Per-key ghost history cannot fix it, because those keys have no ghost history yet. Another lever is needed for first admissions, such as the cap window size or the main-full condition.
