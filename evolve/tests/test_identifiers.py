"""Unit tests for `src.identifiers`."""

from __future__ import annotations

import pytest

from src.identifiers import CycleId, HypothesisId, VariantId


def test_cycle_id_rejects_zero():
    with pytest.raises(ValueError, match="CycleId must be >= 1"):
        CycleId(0)


def test_cycle_id_rejects_negative():
    with pytest.raises(ValueError, match="CycleId must be >= 1"):
        CycleId(-3)


def test_variant_id_rejects_zero():
    with pytest.raises(ValueError, match="VariantId must be >= 1"):
        VariantId(0)


def test_hypothesis_id_rejects_zero():
    with pytest.raises(ValueError, match="HypothesisId must be >= 1"):
        HypothesisId(0)


def test_cycle_id_str_zero_pads_to_four():
    assert str(CycleId(1)) == "0001"
    assert str(CycleId(42)) == "0042"
    assert str(CycleId(9999)) == "9999"


def test_variant_id_str_prefixes_v_and_pads_to_three():
    assert str(VariantId(1)) == "v001"
    assert str(VariantId(17)) == "v017"


def test_hypothesis_id_str_prefixes_H_and_pads_to_two():
    assert str(HypothesisId(1)) == "H01"
    assert str(HypothesisId(12)) == "H12"


def test_cycle_id_round_trip():
    for v in (1, 7, 99, 1234):
        assert CycleId.from_str(str(CycleId(v))) == CycleId(v)


def test_variant_id_round_trip():
    for v in (1, 7, 99):
        assert VariantId.from_str(str(VariantId(v))) == VariantId(v)


def test_cycle_id_from_str_rejects_wrong_length():
    with pytest.raises(ValueError, match="malformed cycle id"):
        CycleId.from_str("001")  # too short
    with pytest.raises(ValueError, match="malformed cycle id"):
        CycleId.from_str("00001")  # too long


def test_cycle_id_from_str_rejects_non_digit():
    with pytest.raises(ValueError, match="malformed cycle id"):
        CycleId.from_str("00a1")


def test_cycle_id_from_str_rejects_zero_padded_zero():
    # "0000" parses to 0, which fails the >= 1 invariant.
    with pytest.raises(ValueError, match="CycleId must be >= 1"):
        CycleId.from_str("0000")


def test_variant_id_from_str_requires_v_prefix():
    with pytest.raises(ValueError, match="malformed variant id"):
        VariantId.from_str("001")


def test_variant_id_from_str_rejects_wrong_length():
    with pytest.raises(ValueError, match="malformed variant id"):
        VariantId.from_str("v01")  # too short
    with pytest.raises(ValueError, match="malformed variant id"):
        VariantId.from_str("v0001")  # too long


def test_variant_id_from_str_rejects_non_digit():
    with pytest.raises(ValueError, match="malformed variant id"):
        VariantId.from_str("v0a1")


def test_cycle_id_cold_start_is_one():
    assert CycleId.cold_start() == CycleId(1)


def test_cycle_id_next_increments():
    assert CycleId(1).next() == CycleId(2)
    assert CycleId(99).next() == CycleId(100)


def test_cycle_id_prev_decrements():
    assert CycleId(2).prev() == CycleId(1)


def test_cycle_id_prev_at_one_raises():
    with pytest.raises(ValueError, match="no cycle precedes"):
        CycleId(1).prev()


def test_cycle_id_is_frozen():
    cid = CycleId(1)
    with pytest.raises(Exception):
        cid.value = 2  # type: ignore[misc]


def test_cycle_id_is_orderable():
    assert CycleId(1) < CycleId(2)
    assert sorted([CycleId(3), CycleId(1), CycleId(2)]) == [CycleId(1), CycleId(2), CycleId(3)]
