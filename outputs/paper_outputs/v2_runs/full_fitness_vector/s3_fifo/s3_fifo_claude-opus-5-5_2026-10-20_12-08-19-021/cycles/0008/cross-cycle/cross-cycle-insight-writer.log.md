# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-opus-5-5)
- workspace: /home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0008/cross-cycle
- allow_edits: True
- started: 2026-10-20T14:39:50+00:00

## timeline

### 14:40:16 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/s3_fifo_2026-10-20_12-08-19-021/cycles/0008/cross-cycle/cross-cycle-insights.md",
  "content": "# Cross-Cycle Insights (through cycle 0008)\n\n## Recurring wins\n\n- **The adaptive small/main split is the durable policy core.** It has held up for eight cycles. Static splits (0003 10/90, 0004 frozen 10/90), ARC-direction rewrites and wholesale policy replacements all lost to it, and it soaks up perturbations to other levers.\n- **Hit-rate moves only through the policy's bounds or depth, never through trigger rates.** Two levers have worked: the lower floor clamp (`capacity/64`, cycle 0005, reproduced in 0006) and retention depth (`MAX_FREQ` 3→7, cycle 0008). Both gains were small but real.\n- **Policy and layout levers are independent and stack.** The floor's gain reproduced exactly on another layout, and the layout refactors kept policy unchanged. Winning levers can be combined across lineages.\n- **For `get` latency, touch only memory the probe already reads.** Three layout changes followed this rule and won: hashed indexing replacing linear scans (0002), moving cold fields such as the key out of the hot array behind a narrow index (0006), and bit-packing `freq` into the index bucket (0008). The current best layout is the narrow `(K, u32)` index with packed `freq`, an 8-byte `Entry`, and keys in a separate array.\n- **Most `put` gains that hold up trace to removed machine work.** Examples are dropping clones and `retain` compaction (the fingerprint ring, 0003), replacing modulo with a compare, and inlining eviction (0004). Count the hashes and clones in the parent before claiming them as a lever.\n\n## Recurring failures\n\n- **Making the ghost path stronger backfires.** This has happened three times: ghosting main victims (0002), readmitting with extra `freq` credit (0005), and growing the ghost to full capacity (0008). Each one readmits keys whose reuse distance is too long and pushes out better residents. Shrinking the ghost (0004) was roughly neutral because the adaptation absorbed it.\n- **Trigger-rate tuning does nothing on workload-a.** Damping grow-small (0006) and doubling the shrink step (0007) were both inert. Loosening the graduation threshold (0002) was flat as well.\n- **Two-lever variants can't be read.** In 0003 (H01 and H03), and in 0004 when ghost size was tied to the split, there was no way to attribute the losses. Pin the ghost size whenever the split is touched.\n- **The wrong kind of layout change hurts `get`:**\n  - An inline payload in the probe table makes buckets wide and not a power of two, and slowed `get` in 0005 and 0006.\n  - Splitting fields that are read together into parallel arrays, like `freq` in 0007, adds a second access on every hit.\n- **Micro-trims below the noise floor are wasted cycles.** Store elision on `get` (0007) is the example.\n- **Measurement has blind spots:**\n  - Credit for `get` latency is unreliable when the `get` path is byte-identical. This happened repeatedly, and drift reached `put` in 0007.\n  - Eviction work on the `put` side leaks into `get` in the mixed benchmark.\n  - `metadata-size` counts only `Entry`, so it misses ghost heap and parallel arrays.\n- **The infrastructure has failed to deliver data:**\n  - Builds failed on Maven 429s in 0001.\n  - The settled `small_size` figure from the split reports has never reached an insight in eight cycles.\n\n## Open questions\n\n- **Where does `small_size` settle, and does the floor bind (`floor_blocks`)?** The loop has to copy the split-report stdout into eval artifacts the insight writers can read.\n- **Can the `MAX_FREQ` hit-rate gain be kept with a cheaper main drain?** Candidates are halving decay instead of decrementing, or a cap of 4–5 packed as 3 bits (a 29-bit slot id makes room).\n- **Would a stricter readmission rule help where every stronger one hurt?** For example, readmit only on a second ghost hit, or only while small is at its floor.\n- **Does a niche-free `keys: Vec<K>` cut the real per-slot footprint?** Liveness would be tracked through `free`.\n- **Are the recent `put` gains (0004 v003, 0008 v001) real?** That needs same-run parent-vs-child benchmarks.\n- **Is the upper clamp (`capacity/2`) a live bound,** the way the floor turned out to be?\n\n## Per-policy notes\n\n- **S3-FIFO with adaptive split (incumbent lineage):** two-touch (`freq > 1`) graduation, ghost-hit shrink, a \"main full on graduation\" grow trigger, and clamps. The balance point appears to be set by the ghost-hit shrink side and the bounds. The current parent is 0008 v001: the `/64` floor, a fingerprint-ring ghost, the narrow index with packed 2-bit `freq`, and an 8-byte `Entry`.\n- **Static-split S3-FIFO (paper 10/90, `freq >= 1`):** lost in 0003, and the frozen 10/90 with `freq > 1` lost again in 0004. Revisit it only as a control: a fixed split near the settled value, with the ghost pinned.\n- **ARC-style two-ghost adaptation:** lost in 0003. Its extra ghost traffic pushed the split in harmful directions. Dropped.\n- **Ghost structure:** the fingerprint ring works as a policy knob as well as plumbing. Its holes shorten the ghost's effective memory, which helped slightly. Ghost size and contents feed the adaptation directly.\n- **CLOCK main queue:** deeper frequency caps help hit-rate but lengthen drains. Any cost to `put` from the main walk shows up in `get` too.\n"
}
```

### 14:40:23 thinking



### 14:40:23 chat

I've written `cross-cycle-insights.md` in the cycle 0008 cross-cycle directory, built from the ledger digest for cycles 0001 to 0008, with the four required sections in order.

- **Recurring wins:** The adaptive small/main split has beaten every replacement for eight cycles. Hit-rate has only moved when the floor clamp or the frequency cap (`MAX_FREQ`) changed, never when trigger rates were tuned. Changes to `get` win when they only touch memory the lookup already reads; the clearest case is packing `freq` into the index.
- **Recurring failures:** Making the ghost path stronger has backfired three times. Tuning trigger rates did nothing. Variants that changed two levers at once couldn't be interpreted. Two layout changes hurt `get`: inline payloads and splitting fields that are read together into separate arrays. Latency results whose `get` code was unchanged look like noise, and `metadata-size` misses ghost memory.
- **Infrastructure gaps:** The settled `small_size` figure has never reached an insight writer in eight cycles, and parents and children are still not benchmarked back-to-back in the same run. Until both are fixed, latency credit and the split's behaviour stay unconfirmed.
- **Open questions:** Whether the frequency-cap gain can survive a cheaper eviction drain; whether stricter readmission helps where every stronger version hurt; whether the upper split clamp also limits anything; and a smaller memory layout for the key array.
- **Per-policy notes:** The incumbent adaptive S3-FIFO lineage, with cycle 0008's v001 as the current parent. Short notes on the static-split and ARC-style variants that lost, the fingerprint-ring ghost, and the cost of deeper caps on the main queue.


---

- finished: 2026-10-20T14:40:24+00:00
- duration: 33.99s
- status: ok
- input_tokens: 4
- output_tokens: 2724
- cache_read_tokens: 41681
- cache_write_tokens: 15131
- cost_usd: 0.18388
