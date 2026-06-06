# Variant v003 Insight

## Helped

- hit-rate: Gating fresh probationary promotions behind `PROMO_THRESHOLD=8` reduced churn in the protected segment by preventing items that accumulated access counts primarily during cold-start from displacing stable residents, while the `was_protected` bypass ensured that items demoted from protected solely by overflow pressure were re-admitted immediately on their next access.

## Hurt

## Takeaway

Pairing an aggressive promotion threshold with a demotion-history bypass flag produces a small but real hit-rate gain on this Zipfian trace: the threshold keeps borderline-hot items from polluting the protected segment, and `was_protected` prevents the same gate from penalizing genuinely hot items that were pushed out by capacity overflow rather than by low frequency. The modesty of the gain suggests the trace's hot set separates cleanly enough that most items either exceed the threshold comfortably or benefit from the bypass, leaving only a thin band of borderline items that the threshold actually blocks — future leverage is more likely in admission or decay logic than in further threshold tuning.
