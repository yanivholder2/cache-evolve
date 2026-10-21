# Variant v001 Insight

## Helped

- (none): the only measured dimension, hit-rate, did not improve over the parent.

## Hurt

- hit-rate: Giving ghost-admitted entries in main one extra reinsertion at freq 0 keeps some entries that are not being re-referenced, and they take space from entries that would be. Hit-rate came out slightly below the parent's.

## Takeaway

A ghost hit shows that an entry was reused once while it sat in the ghost queue. That does not mean the entry deserves a second life in main. The parent's freq-based reinsertion already filters out entries that stop being reused, so a one-time grace pass for ghost-admitted entries mostly delays eviction of dead entries. Protection of ghost-hit entries would need to depend on observed reuse after admission, not on how the entry was admitted. I read this from the hypothesis and the fitness vectors only. I did not diff the overlay sources.
