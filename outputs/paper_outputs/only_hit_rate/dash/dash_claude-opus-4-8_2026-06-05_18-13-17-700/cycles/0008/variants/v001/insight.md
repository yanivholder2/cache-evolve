# Variant v001 Insight

## Helped

- (none): No dimension improved over the parent — the deeper cascade did not convert overflow drops into hits the way the single hop's "land in a free alternate" did.

## Hurt

- hit-rate: Replacing the single non-evicting hop with a 3-hop cascade that displaces each full bucket's LRU resident churns the stash: to keep one overflow victim alive it evicts up to three sitting residents and still drops one when the budget runs out, so it ejects keys that were nearer to promotion in exchange for a victim that is often colder, netting a small loss.

## Takeaway

On top of the protected-tier winner, cascade *depth* is actively worse than the parent's single non-evicting hop: the parent only relocated a victim when a second home had room (no eviction), whereas this cascade pays for extra dwell-time by displacing established stash residents across multiple hops. Because the two-speed promotion already retains warm keys, the residents the cascade evicts are more valuable than the cold victim it rescues — depth that won on the clean baseline inverts here, confirming the cross-cycle question with a negative answer rather than the predicted compounding gain.
