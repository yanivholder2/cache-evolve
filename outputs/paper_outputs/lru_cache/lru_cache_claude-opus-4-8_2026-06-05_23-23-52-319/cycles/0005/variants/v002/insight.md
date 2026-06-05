# Variant v002 Insight

## Helped

- hit-rate: Rebuilding the policy as canonical LIRS — classifying residents by inter-reference recency via stack S and re-admitting short-reuse-distance blocks from non-resident HIR metadata — lifted hit-rate on the LIRS-derived workload-a trace, the trace family this policy was designed for.

## Hurt

- (none): hit-rate is the only measured dimension, and the added stack/Q/non-resident bookkeeping carries no fitness cost.

## Takeaway

Inter-reference recency is a real, distinct workload signal that SLRU's "protect every first hit" promotion cannot capture: distinguishing low-IRR blocks (kept as LIR) from one-shot floods (held only as bounded non-resident metadata for reuse detection) is what raised hit-rate. The genuine LIRS rebuild beat the marginal ghost-probe re-admission of prior SLRU variants because the eviction priority itself, not just admission, was reorganised around measured reuse distance.
