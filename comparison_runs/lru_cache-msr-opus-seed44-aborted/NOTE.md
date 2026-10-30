# Aborted run (kept for completeness)

OpenEvolve seed 44 lost its scratch working directory in %TEMP% (removed by something
outside the run) after iteration 14; iterations 15-30 all failed with
`[Errno 2] No such file or directory` and never reached the model. Only 14 candidates
were evaluated (13 passed gates, best score 0.0485). Not comparable to the other
30-candidate runs; seed 44 was re-run in `lru_cache-msr-opus-seed44` after the fix in
`comparison/claude_code_llm.py` (scratch dir is recreated on every call).
