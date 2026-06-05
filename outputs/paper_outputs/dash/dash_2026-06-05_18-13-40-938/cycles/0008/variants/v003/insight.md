# Variant v003 Insight

## Helped

- **hit-rate**: Gating tier-skip to proven stash residents (access_count >= 2) allows fast-tracked promotion of frequently-reused items while maintaining hit-rate safety through the skip_history_flag recovery path.

## Hurt

(None)

## Takeaway

Controlled tier-skip can coexist with demotion recovery when the skip is conditional on stash tenure (proven by 2+ accesses in stash) and when eviction enforces a mandatory target-tier re-entry via the skip_history_flag. This variant validates that the "unvetted hot-tier residents" problem that broke tier-skip in earlier cycles (0004, 0006) is preventable by structuring the recovery mechanism into the skip decision itself—items that earned their skip get a second chance in target before final stash demotion, reconciling acceleration with safety.
