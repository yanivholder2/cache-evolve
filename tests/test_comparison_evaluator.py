"""The OpenEvolve evaluator's gate-feedback toggle (comparison/evaluator.py)."""

import pytest

pytest.importorskip("openevolve")

from comparison import evaluator  # noqa: E402
from comparison.common import GATE_FEEDBACK_ENV, FAILED_SCORE, Outcome, Setup  # noqa: E402


@pytest.fixture
def failing_eval(tmp_path, monkeypatch):
    program = tmp_path / "program.rs"
    program.write_text("fn main() {}", encoding="utf-8")
    monkeypatch.setenv("COMPARISON_LOG_DIR", str(tmp_path / "log"))
    monkeypatch.setenv("COMPARISON_EVOLVE_FILE", "src/lru_cache/mod.rs")
    monkeypatch.setenv(*next(iter(Setup("lru_cache").to_env().items())))
    monkeypatch.setattr(
        evaluator, "evaluate_in_tempdir",
        lambda *a, **k: Outcome(False, FAILED_SCORE, "gate:build", "error[E0425]: cannot find value"),
    )
    return program


def test_full_feedback_returns_rejection_text(failing_eval, monkeypatch):
    monkeypatch.setenv(GATE_FEEDBACK_ENV, "full")
    result = evaluator.evaluate(str(failing_eval))
    assert result.artifacts["failure_stage"] == "gate:build"
    assert "E0425" in result.artifacts["failure_detail"]


def test_default_is_full_feedback(failing_eval, monkeypatch):
    monkeypatch.delenv(GATE_FEEDBACK_ENV, raising=False)
    assert "failure_detail" in evaluator.evaluate(str(failing_eval)).artifacts


def test_no_feedback_is_a_binary_filter(failing_eval, monkeypatch):
    monkeypatch.setenv(GATE_FEEDBACK_ENV, "none")
    result = evaluator.evaluate(str(failing_eval))
    assert result.artifacts == {}
    assert result.metrics["combined_score"] == FAILED_SCORE
    assert result.metrics["passed"] == 0.0
