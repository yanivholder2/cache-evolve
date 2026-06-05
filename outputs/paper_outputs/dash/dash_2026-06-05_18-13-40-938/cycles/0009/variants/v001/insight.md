# Variant v001 Insight

## Helped

- hit-rate: Tier simplification (Stash + unified Hot instead of Stash + Target + Probing) with all 28 buckets allocated to Hot and direct stash→Hot promotion eliminates intermediate filtering overhead, enabling breakthrough past the 0.684 ceiling that persisted across prior routing interventions.

## Hurt

## Takeaway

Reducing the three-tier hierarchy to a simpler two-tier architecture (Stash + unified Hot) with direct promotion and back-insertion (MRU) demotion achieves the first measurable improvement over the 0.684 ceiling. This confirms the prior insight that the bottleneck was not in *how* items route between tiers, but in the fundamental tier structure itself—having fewer intermediary tiers and more flexible capacity allocation in the hot tier drives better hit rates.
