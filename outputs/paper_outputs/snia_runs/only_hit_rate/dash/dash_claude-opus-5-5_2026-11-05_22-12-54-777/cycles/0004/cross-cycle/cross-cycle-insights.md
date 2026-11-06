## Recurring wins

- Single, isolated changes on top of a clear parent give readable results. Bundled changes (cycle 0002 v001) left their regressions unattributable.
- The 24/8 normal/stash split (cycle 0001) is a stable base. Every later winner was built on it, and it helped most at the large size.
- The frequency-aware 8-slot LRU-tail victim scan with halve-on-cap ageing, counting stash hits (cycle 0002), is the strongest lever so far. It improved all four cells, and it is the only kind of change that has moved the small size.
- Changes that act inside a bucket (victim choice) or at admission move the small cache and hm_0. Topology changes do not.
- Admission-side memory (the cycle 0004 ghost ring with re-admit credit) is the first lever to reach hm_0's long-gap reuse. Recency-position changes never did.
- hm_0 is the most responsive trace in both directions. It is the best place to detect a signal early.

## Recurring failures

- MRU-end promotion has hurt web_0 in cycles 0001, 0002 and 0003, the last time on its own, so MRU landing itself is the cause. Once the tail scan exists, MRU landing also moves new arrivals out of the scan window. Front landing is part of how the filter works.
- Insert-position changes in general (MRU promotion, gated demotion) are redundant or harmful once eviction reads the frequency counter.
- Normal/stash ratio sweeps are exhausted. The split peaks at about 3/4. Shrinking the stash starves promotion on hm_0, because with admit-to-stash the stash is the only way into the protected tier.
- Fixed tail-window sweeps are exhausted. Window 4 lost the filter, and window 16 became coarse LFU and lost recency protection.
- Faster ageing (a lower cap, or a decrement on every eviction) drains counters before they can protect anything. It is worst at small-size hm_0, where items promoted to Front get aged back to 0 inside the window.
- Per-op-time and metadata-size have gone unmeasured for four straight cycles, even though every hypothesis asked for them. Cost-reduction variants cannot be judged, and the costs of the scan and ghost ring on every put are unknown.

## Open questions

- Does a larger ghost ring, or a stronger ghost-hit bonus (tested as separate variants), extend the hm_0 gain, and what does it cost on the admission path?
- Can window width or ageing rate scaled per bucket type (stash vs normal) or by cache size capture the small-hm_0 frequency preference without losing recency elsewhere?
- Would slower-than-halving ageing, or exempting newly promoted items from ageing, help, given the coupling between Front landing and counters?
- Is there any lever that moves web_0? Nothing tried so far has a scan-heavy win.
- Which part of cycle 0003's cheaper config (window vs cap) caused the small-hm_0 loss? This is now low priority.
- What are the actual per-op-time and metadata costs of the 8-slot scan, the halving, and the ghost ring?

## Per-policy notes

- Baseline (target/probing protected tier): the protected tier gives almost no protection. Promoted items were either evicted quickly (Front landing) or crowded out (small normal tier).
- 24/8 topology: this is the parent of all later work. It helps at the large size and does little at the small size on its own.
- Frequency-aware tail victim (window 8, halve-on-cap, stash hits counted): it gains the most at small-size hm_0. Its window, ageing rate and insert positions all sit at local optima, so further tuning of these parameters is spent.
- Ghost-credited admission (64-entry ghost ring per segment, counter 1 on re-admit): this is the current parent. It gives a small gain on hm_0 at both sizes and leaves web_0 flat. It only reaches reuse gaps just past stash residency, and its one-step bonus is fragile under ageing.
