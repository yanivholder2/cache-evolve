# Variant v003 Insight

## Hurt

- **hit-rate**: Bypassing hotness filtering when stash occupancy exceeds 60% promotes non-hot items to probing, increasing eviction churn; these items quickly cascade back through target to stash, creating instability rather than relief.

## Takeaway

Stash backpressure relief via unconditional promotion fails because it conflates two problems: occupancy (space full) and item fitness (worth keeping hot). Forcing non-hottest target residents into probing doesn't reduce cascade pressure—it redistributes it, creating churn in the probing tier and evicting actually hot items. The policy needs either a better condition (e.g., promoting only if item will survive probing eviction) or a different relief mechanism (e.g., stash-to-probing threshold adjustment rather than target-bypass).
