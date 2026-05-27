# Token-frugal principle: anything not requiring LLM judgment is a script

The evolve framework will run for hundreds of Cycles over long timescales. If
the LLM does mechanical work — parsing the Insight ledger, computing the Pareto
front, filtering simulator logs, assembling its own context — runs become
prohibitively expensive and the LLM's context bloats with material the
framework could derive deterministically. We commit architecturally to a split:
**the LLM reserves itself for genuine judgment** (hypothesizing, code-writing,
smoke-log review, cycle insight writing); **everything else is a framework
module in `src/evolve/`** (ledger reads/appends, leaderboard computation,
log filtering, context bundling, cycle sealing).

## Considered options

- **LLM does everything.** The most natural-feeling "agentic" architecture, but
  token-expensive and context-bloating at scale. Rejected.
- **Hybrid with no enforcement.** Allow the agent to invoke scripts ad hoc but
  not require it. In practice the agent re-reads files to "verify" and the
  benefit erodes. Rejected.
- **Explicit token-frugal split with named scripts.** Mechanical operations
  have a script with a fixed contract; the LLM never substitutes. Chosen.

## Consequences

- Module proliferation in `src/evolve/` is expected and correct, not
  evidence of bloat.
- Reviewing a Cycle later is auditable: the AI's tool calls are bounded to a
  small set of named verbs plus file reads/writes.
- If a future change tempts a contributor to "just have the AI do it," that is
  a v1 bug, not a v1.1 convenience.
- Adding a new mechanical operation = adding a new script, not extending an
  agent prompt.
