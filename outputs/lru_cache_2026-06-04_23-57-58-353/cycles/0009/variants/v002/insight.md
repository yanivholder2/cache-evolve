# Variant v002 Insight

## Helped

- hit-rate: Adding a clamped visit counter (0–4) as the primary eviction key lets the cache distinguish entries that have been accessed many times over their lifetime from entries that were merely accessed twice in quick succession — a distinction pure IRR cannot make because it is a two-point snapshot with no memory of earlier accesses.

## Hurt

_(none — only one fitness dimension is tracked)_

## Takeaway

On a LIRS-structured trace, IRR alone leaves a gap: two entries can share the same low IRR yet differ greatly in how often they have been accessed overall. The coarse visit counter fills that gap by demoting recency-but-not-frequency entries before touching entries with accumulated visits. Halving visits on protected-to-probationary demotion is what makes the signal durable across segment transitions — a previously warm entry arrives in probationary with non-zero visits and survives over cold newcomers at visits=0, recovering the frequency-decay benefit that made earlier cycle/0006 variants effective without unbounded counter accumulation.
