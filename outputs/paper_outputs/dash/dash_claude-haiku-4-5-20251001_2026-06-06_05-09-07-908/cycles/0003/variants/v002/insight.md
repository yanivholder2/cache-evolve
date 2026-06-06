# Variant v002 Insight

## Helped

- **per-op-time**: Frequency-gated promotion filters out short-lived false-positives by requiring `lfu_counter >= 2` before advancing from target to probing, reducing unnecessary promotions that trigger costly back-insert operations and demotion cascades (122 ns → 118 ns, ~3.3% improvement).

## Hurt

## Takeaway

Gating target→probing promotion on frequency evidence (two accesses as the hottest item) reduces latency without sacrificing hit-rate. By filtering false-positive promotions, the probing tier contains only genuinely hot items, lowering churn while maintaining the quality of cached residents. This demonstrates that frequency-based selectivity in multi-tier promotion can improve operational efficiency without compromising cache effectiveness.
