# Variant v002 Insight

## Helped

- hit-rate: Admitting stash→target promotions at `InsertPosition::Front` (cold) while keeping cascades and target→probing at Back gave a small gain over v001. One-time stash entries now sit at the eviction end of the target bucket, so scan-like or one-touch keys leave sooner and do not displace re-referenced items. The gain is small enough that it may be within trace-level noise. (Based on the hypothesis; the overlay source was not readable in this session.)

## Hurt

- per-op-time: Both get and put latencies rose against v001, even though the change only flips one enum argument. The likely cause is that cold-admitted items are evicted and re-fetched more often, which brings more promote/demote/cascade work into the hot path. Measurement noise may also contribute, as in v001.

## Takeaway

Making the first admission cold while keeping demotions MRU is a free change in metadata and gives a slight hit-rate gain. It appears to trade that gain for a small latency cost. Position gains from v001 do stack, but weakly. Insert position alone looks close to exhausted as a lever, and the latency delta should be re-measured before it is trusted.
