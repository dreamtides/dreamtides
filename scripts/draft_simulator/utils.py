"""Shared utility functions for the draft simulator.

Small helpers used across multiple modules. Stdlib-only, no external
dependencies.
"""


def argmax(values: list[float]) -> int:
    """Return the index of the maximum value."""
    best_index = 0
    best_value = values[0]
    for i in range(1, len(values)):
        if values[i] > best_value:
            best_value = values[i]
            best_index = i
    return best_index
