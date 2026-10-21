# Variant v001 Insight

## Helped

## Hurt

## Takeaway

The ghost ring was built as the hypothesis specified. Evicted victims' hashes go into a FIFO ring the size of capacity, and a ghost hit can now win an exact tie at an estimate of 2 or more. Even so, workload-a's hit-rate did not move. The most likely reason is that the new admission path almost never fires. The sketch never ages its u32 counters, so on a long trace estimates keep climbing and an exact `incoming_freq == victim_freq` tie becomes rare. On top of that, a returning ghost key keeps collecting sketch increments while it is out of the cache, so it usually either beats the min-of-8 victim outright or still falls short. Only the narrow equal case was changed, and it adds nothing that strict `>` didn't already allow. Recency evidence such as a ghost hit can only matter if it changes outcomes beyond exact ties. Two ways to do that: let a ghost hit move the comparison by a margin (for example, admit when `incoming_freq + k >= victim_freq`), or bring in counter aging so that ties at low counts happen at all.
