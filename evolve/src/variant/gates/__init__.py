"""Gate plug-ins for the per-variant pipeline.

`GATES` is the ordered tuple the runner iterates. Adding a gate is one
new module + one line here; removing is the reverse.
"""

from __future__ import annotations

from src.variant.gates._protocol import Gate, GateOutcome
from src.variant.gates.build_gate import BuildGate
from src.variant.gates.contract_gate import ContractGate
from src.variant.gates.invariant_gate import InvariantGate

GATES: tuple[Gate, ...] = (BuildGate(), ContractGate(), InvariantGate())

__all__ = ["Gate", "GateOutcome", "GATES", "BuildGate", "ContractGate", "InvariantGate"]
