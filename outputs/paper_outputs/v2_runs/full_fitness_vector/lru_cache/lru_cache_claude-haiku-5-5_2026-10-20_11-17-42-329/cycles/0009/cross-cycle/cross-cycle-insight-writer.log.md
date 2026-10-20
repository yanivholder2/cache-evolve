# Agent log: cross-cycle-insight-writer

- adapter: ClaudeSdkAdapter(model=claude-haiku-5-5)
- workspace: /home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0009/cross-cycle
- allow_edits: True
- started: 2026-10-20T12:03:18+00:00

## timeline

### 12:03:28 thinking



### 12:03:34 tool: Write

```json
{
  "file_path": "/home/user/cache-evolve/outputs/lru_cache_2026-10-20_11-17-42-329/cycles/0009/cross-cycle/cross-cycle-insights.md",
  "content": "## Recurring wins\n\n- Cheap hashing on the key map and admission sketch lowers per-operation cost without touching policy. A zero-state map hasher and a single shared sketch hash both helped, while extra finish-step mixing gave no benefit.\n- Narrowing arena links and list pointers to `u32` with a sentinel is a policy-neutral way to shrink nodes. Node-size wins have not cost hit-rate, though they do not speed up `get`.\n- Keeping the rescued protected head in place, rather than rotating it on the protected-side demotion path, removes per-operation list surgery. The price is a small hit-rate give-back.\n- Bounding the second-chance scan trades a small hit-rate loss for a more predictable miss-path tail.\n- Removing a hash from the read path cleanly lowers read cost. This is the admission sketch change, which gave up a small amount of hit-rate.\n- Consolidating masked index reads into one inlined helper recovers cost on a packed-flag layout.\n\n## Recurring failures\n\n- Ghost-ring admission has not held. Promoting ghost hits to protected lost hit-rate, and promoting them to probation also lost hit-rate on workload-a at 2048. The ghost's per-miss scan and hashing added miss-path cost either way. Treat it as dropped.\n- Extra pointer work or list surgery on shared hit and overwrite paths has repeatedly shown up as per-operation cost.\n- Changes to arena indexing, including hoisted accessors, narrowed casts, and masked reads, have repeatedly left `get` or `put` slower. Each such change has been an inference rather than a confirmed cause.\n- Replacing the segmented structure with CLOCK removed the protection the parent relied on and added per-slot allocation cost.\n- Guarding the reference-bit write, and collapsing the `put` miss path onto a single key lookup, did not reduce cost.\n- Shrinking the protected share lowered hit-rate, so reductions there are unlikely to pay.\n\n## Open questions\n\n- Are the recurring `get` slowdowns from index access real, or are they codegen or layout effects? A codegen-only comparison has not been run.\n- Do the hasher swap, the sketch hash change, and the protected-share effects hold beyond workload-a at 2048? Every result so far is from that one trace and capacity.\n- Is the `put` tail movement seen across many variants real, or run-to-run noise? It has never been separated from noise.\n- Can protection be kept without list surgery on the shared path, for example through a credit or a different recency signal?\n- Several Winner labels do not match the Helped sections of their insights. Fitness vectors should be checked before crediting those variants.\n\n## Per-policy notes\n\n- **Segmented (SLRU) eviction:** The protected share is the main hit-rate lever. Growing it helped on workload-a at 2048 but thinned probation and added `put` miss-path work. Shrinking it hurt. The current range looks like a trade-off. Protection depends on the rescued head actually moving, so a retained flag or two-bit credit recovers only a small share of it.\n- **Second-chance and CLOCK:** A bounded second-chance scan is the clearest per-operation gain in this family. A plain CLOCK ring of counters lost the segmented protection, so it is not a substitute.\n- **Admission sketch (frequency-filtered):** Rejecting one-hit keys at capacity gave the cohort's clearest hit-rate gain. Its per-access hashing was the cost, and a cheaper hash or a read-path removal reduced it. The read-path removal is the open trade-off, since read-only popularity no longer counts.\n- **Ghost ring:** No admission target has held hit-rate on the tested trace. Do not pursue it further without a new signal.\n"
}
```

### 12:03:37 chat

I wrote `cross-cycle-insights.md` to the cycle 0009 cross-cycle output directory. It has the four required sections in order, and I built it only from the inlined ledger digest without reading any prior copy of the file.

The main signals across the nine cycles:

- **Recurring wins:** cheap hashing on the key map and admission sketch, `u32` sentinel links for smaller nodes, keeping the protected head in place to avoid list surgery, and a bounded second-chance scan for a steadier miss-path tail.
- **Recurring failures:** ghost-ring admission, which lost hit-rate whichever segment it promoted into; extra pointer work on shared paths; index-access rewrites that left `get` or `put` slower; the CLOCK rewrite; and shrinking the protected share.
- **Open questions:** whether the `get` slowdowns from index access are real or codegen effects, whether the hasher and sketch results hold beyond workload-a at 2048, and whether the recurring `put` tail movement is noise.
- **Per-policy notes:** grouped under segmented (SLRU), second-chance/CLOCK, the admission sketch, and the ghost ring.

The digest also shows that several Winner labels (cycles 7, 8 and 9) don't match the Helped sections of their insights. I flagged that as an open question rather than crediting those variants. No numbers were invented; everything is stated qualitatively.


---

- finished: 2026-10-20T12:03:37+00:00
- duration: 19.38s
- status: ok
- input_tokens: 4
- output_tokens: 3477
- cache_read_tokens: 41388
- cache_write_tokens: 14943
- cost_usd: 0.005141
