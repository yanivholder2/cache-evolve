# Variant v002 Insight

## Helped

*(none)*

## Hurt

- hit-rate: Reserving ⌈capacity/100⌉ slots as a permanent admission window shrank the main LRU's effective capacity, giving established high-frequency items fewer slots on workload-a, which more than cancelled any benefit from deferring the frequency gate for newcomers.

## Takeaway

The admission window's core assumption — that new items need time to accumulate sketch frequency before facing the gate — does not hold on workload-a: items entering the window have frequency 1 (from the `put` increment) and rarely get a second access before being rotated out, so they lose the window-vs-main comparison anyway. The real cost is structural: permanently reserving ~1% of capacity for a probationary zone reduces the steady-state size of the main LRU, and that capacity loss is felt more acutely than the marginal improvement in newcomer protection.
