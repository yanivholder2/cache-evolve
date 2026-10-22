"""Run OpenEvolve on one cache-evolve subject: `python -m comparison.run_openevolve`."""

from __future__ import annotations

import argparse
import json
import os
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parents[1]))

from comparison.common import REPO_ROOT, default_evolve_file  # noqa: E402

SYSTEM_MESSAGE = """\
You are evolving a cache implementation written in Rust. The program is one source
file of a subject crate; it is built into a native library that the Caffeine cache
simulator drives over JNI against a recorded access trace.

## Objective
Maximise the weighted improvement over the baseline cache across these dimensions:
{objective}
The reported combined_score is that weighted fractional improvement (0 = baseline).

## Hard constraints (a violation scores as a failure)
- The crate must compile, satisfy the `Cache<K, V>` contract, and pass the subject's
  invariant tests.
- Keep every `pub mod ...;` declaration and the public API (names, signatures) that
  the rest of the crate and the JNI boundary use. Only this file may change.
- No new external crates; the standard library only.

## Subject domain notes
{context}

## Baseline note
{insight}
"""


def build_system_message(subject: str, weights: dict[str, float]) -> str:
    subject_dir = REPO_ROOT / "inputs" / "subjects" / subject
    objective = "\n".join(f"- {dim}: weight {w}" for dim, w in weights.items())
    context = (subject_dir / "CONTEXT.md").read_text(encoding="utf-8")
    insight = (subject_dir / "baseline" / "insight.md").read_text(encoding="utf-8")
    return SYSTEM_MESSAGE.format(objective=objective, context=context, insight=insight)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--subject", required=True)
    parser.add_argument("--evolve-file", default=None, help="path relative to the subject crate")
    parser.add_argument("--iterations", type=int, required=True)
    parser.add_argument("--model", required=True)
    parser.add_argument("--provider", choices=["claude_code", "openai"], default="claude_code")
    parser.add_argument("--api-base", default=None, help="OpenAI-compatible endpoint (provider=openai)")
    parser.add_argument("--max-tokens", type=int, default=16000)
    parser.add_argument("--llm-timeout-s", type=int, default=600)
    parser.add_argument("--eval-timeout-s", type=int, default=1800)
    parser.add_argument("--seed", type=int, default=42)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args(argv)

    from openevolve.api import run_evolution
    from openevolve.config import Config, LLMModelConfig
    from comparison.common import load_configs

    evolve_file = args.evolve_file or default_evolve_file(args.subject)
    out = args.out.resolve()
    out.mkdir(parents=True, exist_ok=True)
    os.environ["COMPARISON_SUBJECT"] = args.subject
    os.environ["COMPARISON_EVOLVE_FILE"] = evolve_file
    os.environ["COMPARISON_LOG_DIR"] = str(out)

    cfg = load_configs(REPO_ROOT, subject=args.subject)
    seed_file = REPO_ROOT / "inputs" / "subjects" / args.subject / evolve_file

    config = Config()
    config.max_iterations = args.iterations
    config.random_seed = args.seed
    config.database.random_seed = args.seed
    config.language = "rust"
    config.file_suffix = ".rs"
    config.diff_based_evolution = True
    config.max_code_length = 200_000
    config.checkpoint_interval = 1
    config.log_level = "INFO"
    config.prompt.system_message = build_system_message(args.subject, cfg.selection.weights or {})
    config.evaluator.cascade_evaluation = False
    config.evaluator.parallel_evaluations = 1
    config.evaluator.timeout = args.eval_timeout_s

    model = LLMModelConfig(name=args.model, weight=1.0)
    if args.provider == "claude_code":
        from comparison.claude_code_llm import init_isolated_claude_code_client
        model.init_client = init_isolated_claude_code_client
    else:
        model.provider = "openai"
        if args.api_base:
            model.api_base = args.api_base
    config.llm.models = [model]
    config.llm.evaluator_models = [model]
    config.llm.max_tokens = args.max_tokens
    config.llm.timeout = args.llm_timeout_s
    config.llm.update_model_params(
        {"max_tokens": args.max_tokens, "timeout": args.llm_timeout_s}, overwrite=True
    )

    (out / "run.json").write_text(json.dumps({
        "subject": args.subject, "evolve_file": evolve_file, "iterations": args.iterations,
        "model": args.model, "provider": args.provider, "seed": args.seed,
        "openevolve_version": __import__("openevolve")._version.__version__,
    }, indent=2))

    result = run_evolution(
        initial_program=str(seed_file),
        evaluator=str(Path(__file__).with_name("evaluator.py")),
        config=config,
        iterations=args.iterations,
        output_dir=str(out / "openevolve_output"),
        cleanup=False,
    )
    print(f"openevolve best_score={result.best_score}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
