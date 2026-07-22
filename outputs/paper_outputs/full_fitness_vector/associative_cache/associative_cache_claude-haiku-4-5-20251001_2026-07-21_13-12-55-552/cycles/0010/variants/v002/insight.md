# Variant v002 Insight

## Hurt

- **hit-rate**: Batching tracking at 64-operation boundaries reduces observation density from every access to every 64th access, weakening the insertion_bias calculation's ability to capture fine-grained temporal reuse patterns. The sparser sample causes bias computation to miss locality structure, dropping hit-rate from 71.8% to 70.0%.

- **per-op-time**: The operation_counter increment and modulo check on every operation (lines 81, 123, 127) introduce a measurable per-operation cost that outweighs the savings from deferring the internal tracking logic. Get p50 doubled from 100 ns to 200 ns, and put p99 jumped from 300 ns to 500 ns, indicating the counter arithmetic itself is expensive relative to the deferred timestamp comparisons.

- **metadata-size**: Adding the operation_counter field increased the AssociativeCache struct by 8 bytes (from 80 to 88), a persistent overhead that doesn't disappear after characterization ends.

## Takeaway

Amortizing overhead via operation counting is self-defeating: the counter's per-operation cost (increment + modulo) is paid on every single access and nullifies the savings from reduced tracking frequency. The variant demonstrates that batching is only effective if the *per-operation check cost* is negligible compared to the deferred work; here, the counter is not negligible, and the sparser tracking also degrades the quality of the bias signal. The core issue from v003 (per-operation latency during characterization) cannot be solved by moving the work—it can only be solved by making the work genuinely free or eliminating the cost driver (e.g., removing Instant comparisons entirely or using a lock-free counter that doesn't stall the pipeline).
