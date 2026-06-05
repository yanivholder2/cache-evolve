# Variant v002 Insight

## Helped

## Hurt

- hit-rate: Changing the second-chance promotion from `InsertPosition::Front` to `InsertPosition::Back` placed stash-evicted items immediately at the MRU end of their target bucket, making them instant candidates for probing promotion and displacing genuinely hot items that had earned their target position through repeated access.

## Takeaway

The MRU-wins principle does not extend to second-chance items: they are stash residents displaced by hash collision, not by proven coldness, but they have also not demonstrated hotness. Inserting them at Back gives them immediate probing candidacy on their very next access, which floods the probing tier with unproven items and evicts residents that earned their position through actual repeated use. Front insertion, as used in v001, correctly treats second-chance items as cold probationers that must survive target competition before advancing — the asymmetry between v001 and v002 confirms that entry position at the second-chance recovery site acts as a quality gate, not just a placement detail.
