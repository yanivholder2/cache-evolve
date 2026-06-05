# Variant v003 Insight

## Hurt

- hit-rate: Bypassing the target tier for stash hits breaks the tier-skipping mechanism's synergy with back-insertion demotion strategy; items promoted directly to probing lack the intermediate target evaluation, creating cache pollution from underproven stash hits competing for probing space that should be reserved for tier-climbers.

## Takeaway

Tier-skipping stash hits and back-insertion for demoted items have conflicting assumptions about tier movement. While cycle 0002's tier-skip alone (0.678) and cycle 0003's back-insertion alone (0.6839) each contributed gains, combining them produces worse results (0.6785). The tier-skip strategy requires items to be heavily filtered by intermediate tiers before promotion, but back-insertion reverses this discipline—demoting items back to the stash at MRU position, then accelerating their re-entry directly to probing. This creates two classes of probing residents: earned climbers and fast-tracked re-entries from the stash, undermining the tiered promotion hierarchy that back-insertion was optimizing.
