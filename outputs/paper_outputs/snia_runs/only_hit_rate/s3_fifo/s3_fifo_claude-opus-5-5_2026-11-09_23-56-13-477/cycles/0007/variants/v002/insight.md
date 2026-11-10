# Variant v002 Insight

## Helped

- hit-rate (web_0 @ 131072): This is the only cell that moved the way the hypothesis predicted, and only barely. At this capacity the small queue is big enough that a key can saturate freq during one residence and still be useful afterwards, so giving those keys one protected lap in main kept a few of them past their first trip to the tail.

## Hurt

- hit-rate (hm_0 @ 16384): The parent's recency-gated ghost path helped this cell most, and this variant lost the most here. Quick freq-0 ghost returnees go straight to main, and they are now the first freq-0 entries to reach the tail. The freq-1 lap given to saturated graduates is paid for by evicting exactly the population the parent showed is valuable.
- hit-rate (web_0 @ 16384): Main is small at this size, so every protected lap pushes out a freq-0 entry that would otherwise have had its chance. Keys that saturated in small seem to get re-hit in main anyway, which means the extra lap adds little.
- hit-rate (hm_0 @ 131072): The prediction that "the affected population grows with cache size" did not hold on hm_0. More saturated graduates means more freq-1 entries pressing on the freq-0 tail, and the net effect was a small loss rather than a gain.

## Takeaway

Saturating freq in small does not reliably mean a key needs extra protection in main. Main's own second-chance already rewards keys that keep getting hits, so a head start on admission mostly duplicates that. Its cost falls on the quick ghost returnees that the parent's age gate deliberately admitted. The rationale assumed those returnees were "weakly evidenced", but the parent's result says the opposite. Admission-time freq tweaks in main should not take protection away from that path. A follow-up that wants to reward saturated graduates should do it without moving them ahead of quick ghost returnees, for example by giving both the same starting freq.
