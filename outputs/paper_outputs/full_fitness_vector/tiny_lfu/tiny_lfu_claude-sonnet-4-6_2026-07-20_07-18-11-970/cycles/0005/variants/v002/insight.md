# Variant v002 Insight

## Helped

- `per-op-time`: Replacing `seeded_hash(key, 1)` with the three-instruction `remix(h0)` eliminated the second SipHash invocation on the put path, collapsing put p50 and p99 back to the same levels as get — closing the asymmetry that v001 introduced.

## Hurt

- `hit-rate`: The bijective multiply-xorshift remix provides weaker inter-pair independence than two independently seeded SipHash outputs, producing a marginal erosion in frequency estimation accuracy that is likely at the edge of measurement noise.

## Takeaway

Replacing the second seeded hash with a cheap bijective remix is a clean win for put-path latency: the ALU-only re-mix eliminates the serial hash dependency that caused v001's put to be measurably slower than get. The near-zero hit-rate change confirms that genuine pairwise independence between row groups — which required a full SipHash call in v001 — is more independence than the count-min sketch actually needs for accurate admission decisions on this workload; the weaker structural mixing from the Fibonacci multiply-rotate-XOR is sufficient.
