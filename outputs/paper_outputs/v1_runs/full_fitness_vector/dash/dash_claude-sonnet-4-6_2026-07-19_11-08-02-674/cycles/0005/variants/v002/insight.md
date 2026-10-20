# Variant v002 Insight

## Helped

- **hit-rate**: The vacancy-gated probing re-entry path moves items in probing that have accumulated `lfu_counter >= 2` back into their true target bucket when space exists, recovering target-tier residency for items that were demoted but continued to receive hits — producing a fractional but positive uplift over the parent.

## Hurt

*(no dimensions moved negatively)*

## Takeaway

Applying the vacancy-gated frequency mechanism symmetrically to the probing-hit branch — so that demoted items earning repeated accesses in probing can reclaim a target slot when one is free — confirmed that the direction is safe and marginally beneficial. The gain is small because the joint condition (`lfu_counter >= 2` in probing AND target has vacancy) is rarely satisfied simultaneously on this workload, indicating that most probing residents either fail the frequency threshold or face a full target, so the re-entry path fires infrequently and contributes only a thin incremental lift on top of the parent's already-established target-frequency promotion.
