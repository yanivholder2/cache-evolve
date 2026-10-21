# Variant v003 Insight

## Helped

(No measured dimension improved. The variant adds nothing that should help on its own. Every ghost hit still takes the front-of-main re-admission path from the parent. The only behavioural change is where the small/main split sits over time.)

## Hurt

- hit-rate: H03 adapts the split from the source of each ghost hit, so the direction it moves depends on how ghost hits divide between small-queue rejects (`from_main = false`, grows small) and main victims (`from_main = true`, shrinks small). The parent's insight found that main victims rarely return within the ghost window under `MAX_FREQ = 7`. So small-reject hits most likely outnumber main-victim hits, and the controller pushes `small_size` up from 5% toward the 10% cap, back into the range the earlier static sweep (10%→5%) had already shown to be worse. The eval records no `small_size` trajectory, so this drift is inferred, not observed. A second problem: the "grow" signal is acted on after the fact. A returning small reject is already re-admitted straight into main, so a bigger small queue doesn't recover that hit. It just takes main slots from CLOCK-protected keys. Because eviction is checked against the total, main also gives up residents (with `from_main` ghost entries) as soon as small grows, even when main is under its new target.

## Takeaway

On workload-a, ghost-hit source is a poor control signal for the S3-FIFO split. The ghost is shared and capped at the initial `main_size`, and small rejects dominate it, so the ARC-style counter is biased toward a larger small queue instead of following real phase changes. Before adaptation can pay off, the two hit sources need comparable evidence: separate ghost budgets, or hits weighted by the size of each source (as ARC does with |B1|/|B2|). Without that, a fixed 5% split beats any controller limited to [1%, 10%]. Logging `small_size` over the trace would confirm or rule out the upward drift.
