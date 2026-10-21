# Variant v002 Insight

## Helped

- hit-rate: Essentially unchanged; any movement is within trace noise. The locking change was meant to leave admission semantics alone, and the result is consistent with that. (Based on the hypothesis; the overlay source was not readable in this session.)

## Hurt

- per-op-time: Get latency got worse, which is the opposite of what the hypothesis targeted. Put also got slower even though put was not supposed to change. A shared-lock read path with an upgrade-and-revalidate step on promotion probably costs more than it saves. Hits on this trace often need a state change, so they pay for the read acquire, the drop, the exclusive re-acquire and the re-check. Reader-writer lock bookkeeping is also heavier than a plain exclusive lock when there is little contention. The put regression may be measurement noise or extra lock-state traffic.

## Takeaway

Shared-then-upgrade locking did not pay off in this single-threaded-style benchmark. With little contention, the exclusive lock was already cheap, and the reader-writer overhead plus double acquisition on promoting hits outweighed any parallel-read benefit. Hit-rate and metadata size staying flat shows the change was behaviour-neutral but not a latency win. Locking only helps if the benchmark has concurrent readers, or if most hits can finish without any write.
