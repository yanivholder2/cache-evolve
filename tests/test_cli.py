"""Unit tests for src/evolve/cli.py helpers."""

from __future__ import annotations

import types

import pytest

from evolve.cli import _require_scope_supported
from evolve.config import AuthoringScope


def _stub_subject(extension_points: tuple = ()) -> types.SimpleNamespace:
    return types.SimpleNamespace(name="lru_cache", extension_points=extension_points)


def test_extension_points_scope_rejects_subject_without_extension_points():
    with pytest.raises(SystemExit):
        _require_scope_supported(_stub_subject(()), AuthoringScope.EXTENSION_POINTS)


def test_extension_points_scope_allows_subject_with_extension_points():
    _require_scope_supported(
        _stub_subject(("ep",)), AuthoringScope.EXTENSION_POINTS
    )


def test_full_autonomy_scope_allows_subject_without_extension_points():
    _require_scope_supported(_stub_subject(()), AuthoringScope.FULL_AUTONOMY)
