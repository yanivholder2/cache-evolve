## Recurring wins

- Insertion depth is the only lever with repeated hit-rate gains. New keys placed below the hot head (tail in one form, a midpoint marker in another) produced small gains in several cycles, and the midpoint form is the most consistent.
- Eviction-neutral layout and placement changes (sentinel indices, head-hit relink skip, pre-sized index, hasher swap) hold hit-rate flat against their parent. This is expected behaviour preservation, not yet a demonstrated benefit.
- A frequency signal helps only when it is confined to the LRU end. A small bounded victim window gave a marginal gain, while wider windows lost.

## Recurring failures

- Changes that alter eviction order structurally have lowered hit-rate: segmented recency lists, second-chance referenced bits, and second-hit promotion. Strict recency ordering appears to do useful work on this trace.
- Frequency counters and admission gates have been flat or slightly negative: decaying or capping hit counters, count-min admission gating, and widening the frequency-aware victim window.
- Tuning the segmentation in the downward direction lost: shrinking the protected share lowered hit-rate, so the larger protected share holds re-referenced keys better.
- Lengthening ghost-set retention lost hit-rate, so longer ghost memory admits keys that do not recur.
- Winners have repeatedly been named without a measured gain on the dimension they target, and some were named despite a hit-rate dip. This selection-criterion gap runs from cycle 0001 through cycle 0010.
- Rejections for infrastructure reasons (session limits, missing insight files) carry no strategy signal and should not be counted as failures of their approach.

## Open questions

- Per-op time and metadata size have not been recorded for any variant in any cycle. Every claim about memory or time cost is untested.
- Does the midpoint insertion gain hold on a second trace or another cache size, and how does it change at other marker depths?
- Does the larger protected share hold up above its current setting, or is it already near the best point?
- Did the tail-insertion variants that conflicted (gain, loss, null across cycles 0005 to 0007) differ mainly in insertion depth? Is that conflict now explained?
- What does the ghost-set retention behaviour look like on another trace, now that engagement has been confirmed?
- Does the frequency-aware tail-window gain survive on a second trace, or is it specific to the current workload and size?
- Hypothesis labels (notably H03) have been mapped to different mechanisms across cycles. Which definition should future lineages cite?

## Per-policy notes

- Plain LRU (strict recency): changes that reorder the recency list have lost hit-rate; placement-only changes that keep order hold it flat. Shallower-than-head insertion of new keys is the main candidate for a gain.
- Segmented (probation/protected): the larger protected share outperforms the smaller one. Midpoint insertion gives a small gain on this base. The split alone does not beat the baseline.
- Frequency-aware eviction: a bounded window near the LRU end is the only frequency variant with a marginal gain. Wider scans, counter decay, and capped counters did not help.
- Ghost/admission filters: confirmed to engage once retention is varied, and longer retention lost. Count-min and correlated-reference gates have not produced a measured gain.
- Second-chance and second-hit promotion: both lowered hit-rate and are not supported on this trace.
