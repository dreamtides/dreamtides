"""Configuration system for the draft simulator.

Provides a hierarchical configuration with defaults matching the design
document, TOML/JSON file loading, dot-notation CLI overrides, and
validation. Stdlib-only, no external dependencies.
"""

import copy
import json
import sys
from dataclasses import dataclass, field, fields
from typing import Any, Optional


@dataclass
class DraftConfig:
    """Draft structure parameters."""

    seat_count: int = 6
    round_count: int = 3
    picks_per_round: list[int] = field(default_factory=lambda: [10, 10, 10])
    pack_size: int = 20
    alternate_direction: bool = False
    human_seats: int = 1


@dataclass
class CubeConfig:
    """Cube construction parameters."""

    distinct_cards: int = 360
    copies_per_card: int = 1
    consumption_mode: str = "without_replacement"


@dataclass
class PackGenerationConfig:
    """Pack generation strategy parameters."""

    strategy: str = "seeded_themed"
    archetype_target_count: int = 4
    primary_density: float = 0.5
    bridge_density: float = 0.15
    variance: float = 0.3


@dataclass
class RefillConfig:
    """Pack refill strategy parameters."""

    strategy: str = "no_refill"
    fingerprint_source: str = "pack_origin"
    fidelity: float = 0.7
    commit_bias: float = 0.3


@dataclass
class CardsConfig:
    """Card sourcing parameters."""

    source: str = "synthetic"
    file_path: Optional[str] = None
    rendered_toml_path: Optional[str] = None
    metadata_toml_path: Optional[str] = None
    archetype_count: int = 8
    bridge_fraction: float = 0.15


@dataclass
class AgentsConfig:
    """Agent behavior parameters."""

    policy: str = "adaptive"
    show_n: int = 4
    show_n_strategy: str = "sharpened_preference"
    ai_optimality: float = 0.8
    ai_signal_weight: float = 0.2
    ai_power_weight: float = 0.2
    ai_pref_weight: float = 0.6
    openness_window: int = 3
    learning_rate: float = 3.0
    force_archetype: Optional[int] = None


@dataclass
class ScoringConfig:
    """Deck value scoring parameters."""

    weight_power: float = 0.3
    weight_coherence: float = 0.5
    weight_focus: float = 0.2
    secondary_weight: float = 0.3
    focus_threshold: float = 0.5
    focus_saturation: float = 0.7


@dataclass
class CommitmentConfig:
    """Commitment detection parameters."""

    commitment_threshold: float = 0.15
    stability_window: int = 7
    entropy_threshold: float = 2.0


@dataclass
class MetricsConfig:
    """Metrics engine parameters."""

    richness_gap: float = 0.1
    tau: float = 1.0
    on_plan_threshold: float = 0.3
    splash_power_threshold: float = 0.5
    splash_flex_threshold: float = 0.6
    exposure_threshold: float = 0.5


@dataclass
class RarityConfig:
    """Rarity tier system parameters."""

    enabled: bool = True
    tiers: list[str] = field(default_factory=lambda: ["common", "uncommon", "rare"])
    tier_design_counts: list[int] = field(default_factory=lambda: [180, 120, 60])
    tier_copies: list[int] = field(default_factory=lambda: [3, 2, 1])
    tier_power_ranges: list[list[float]] = field(
        default_factory=lambda: [[0.2, 0.6], [0.4, 0.7], [0.6, 0.9]]
    )
    pack_tier_slots: list[int] = field(default_factory=lambda: [12, 5, 3])


@dataclass
class SweepConfig:
    """Sweep execution parameters."""

    runs_per_point: int = 1000
    base_seed: int = 42
    seeding_policy: str = "sequential"
    trace_enabled: bool = False
    axes: dict[str, list[Any]] = field(default_factory=dict)


@dataclass
class SimulatorConfig:
    """Top-level configuration containing all sections."""

    draft: DraftConfig = field(default_factory=DraftConfig)
    cube: CubeConfig = field(default_factory=CubeConfig)
    pack_generation: PackGenerationConfig = field(default_factory=PackGenerationConfig)
    refill: RefillConfig = field(default_factory=RefillConfig)
    cards: CardsConfig = field(default_factory=CardsConfig)
    agents: AgentsConfig = field(default_factory=AgentsConfig)
    scoring: ScoringConfig = field(default_factory=ScoringConfig)
    commitment: CommitmentConfig = field(default_factory=CommitmentConfig)
    metrics: MetricsConfig = field(default_factory=MetricsConfig)
    rarity: RarityConfig = field(default_factory=RarityConfig)
    sweep: SweepConfig = field(default_factory=SweepConfig)


SECTION_NAMES: list[str] = [
    "draft",
    "cube",
    "pack_generation",
    "refill",
    "cards",
    "agents",
    "scoring",
    "commitment",
    "metrics",
    "rarity",
    "sweep",
]

_SECTION_CLASSES: dict[str, type] = {
    "draft": DraftConfig,
    "cube": CubeConfig,
    "pack_generation": PackGenerationConfig,
    "refill": RefillConfig,
    "cards": CardsConfig,
    "agents": AgentsConfig,
    "scoring": ScoringConfig,
    "commitment": CommitmentConfig,
    "metrics": MetricsConfig,
    "rarity": RarityConfig,
    "sweep": SweepConfig,
}


def clone_config(cfg: "SimulatorConfig") -> "SimulatorConfig":
    """Create a deep copy of a SimulatorConfig with all sub-sections copied."""
    return copy.deepcopy(cfg)


def config_to_sorted_pairs(
    cfg: "SimulatorConfig",
    exclude_sections: Optional[set[str]] = None,
) -> list[str]:
    """Serialize all config parameters as sorted "section.param=value" strings.

    Iterates over all sections and their dataclass fields using introspection.
    Optionally excludes named sections (e.g., "sweep").
    """
    excluded = exclude_sections or set()
    pairs: list[str] = []
    for section_name in SECTION_NAMES:
        if section_name in excluded:
            continue
        section_obj = getattr(cfg, section_name)
        for f in sorted(fields(section_obj), key=lambda x: x.name):
            value = getattr(section_obj, f.name)
            pairs.append(f"{section_name}.{f.name}={value}")
    pairs.sort()
    return pairs


def _coerce_value(current: Any, raw: str) -> Any:
    """Coerce a string CLI value to match the type of the current value."""
    if isinstance(current, bool):
        return raw.lower() in ("true", "1", "yes")
    if isinstance(current, int):
        return int(raw)
    if isinstance(current, float):
        return float(raw)
    if isinstance(current, list):
        return json.loads(raw)
    if isinstance(current, dict):
        return json.loads(raw)
    if current is None:
        if raw.lower() in ("none", "null"):
            return None
        try:
            return int(raw)
        except ValueError:
            pass
        try:
            return float(raw)
        except ValueError:
            pass
        return raw
    return raw


def load_config(
    config_path: Optional[str] = None,
    overrides: Optional[list[str]] = None,
) -> SimulatorConfig:
    """Load configuration from optional file and apply CLI overrides.

    Supports TOML (Python 3.11+ via tomllib) and JSON config files.
    CLI overrides use dot notation: section.param=value.
    """
    cfg = SimulatorConfig()

    if config_path is not None:
        raw = _load_file(config_path)
        _apply_dict(cfg, raw)

    if overrides:
        for override in overrides:
            _apply_override(cfg, override)

    validate_config(cfg)
    return cfg


def _load_file(path: str) -> dict[str, Any]:
    """Load a TOML or JSON configuration file."""
    if path.endswith(".toml"):
        return _load_toml(path)
    return _load_json(path)


def _load_toml(path: str) -> dict[str, Any]:
    """Load a TOML file using tomllib (Python 3.11+)."""
    if sys.version_info < (3, 11):
        raise RuntimeError(
            "TOML config files require Python 3.11+ (tomllib). "
            "Use JSON format or upgrade Python."
        )
    import tomllib

    with open(path, "rb") as f:
        return tomllib.load(f)


def _load_json(path: str) -> dict[str, Any]:
    """Load a JSON configuration file."""
    with open(path, "r") as f:
        return json.load(f)


def _apply_dict(cfg: SimulatorConfig, raw: dict[str, Any]) -> None:
    """Apply a nested dict of values to a SimulatorConfig."""
    for section_name, section_values in raw.items():
        if section_name not in _SECTION_CLASSES:
            continue
        if not isinstance(section_values, dict):
            continue
        section = getattr(cfg, section_name)
        for key, value in section_values.items():
            if hasattr(section, key):
                setattr(section, key, value)


def _apply_override(cfg: SimulatorConfig, override: str) -> None:
    """Apply a single dot-notation override like 'draft.seat_count=8'."""
    if "=" not in override:
        raise ValueError(f"Invalid override format (expected KEY=VALUE): {override}")

    key, raw_value = override.split("=", 1)
    parts = key.split(".")

    if len(parts) != 2:
        raise ValueError(f"Override key must be section.param (got {key!r})")

    section_name, param_name = parts
    if not hasattr(cfg, section_name):
        raise ValueError(f"Unknown config section: {section_name!r}")

    section = getattr(cfg, section_name)
    if not hasattr(section, param_name):
        raise ValueError(
            f"Unknown parameter {param_name!r} in section {section_name!r}"
        )

    current = getattr(section, param_name)
    setattr(section, param_name, _coerce_value(current, raw_value))


def validate_config(cfg: SimulatorConfig) -> None:
    """Validate configuration constraints. Raises ValueError on failure."""
    errors: list[str] = []

    # picks_per_round length must equal round_count
    if len(cfg.draft.picks_per_round) != cfg.draft.round_count:
        errors.append(
            f"len(picks_per_round)={len(cfg.draft.picks_per_round)} "
            f"!= round_count={cfg.draft.round_count}"
        )

    # picks_per_round must sum to 30
    picks_sum = sum(cfg.draft.picks_per_round)
    if picks_sum != 30:
        errors.append(f"sum(picks_per_round)={picks_sum} != 30")

    # In no-refill mode, pack_size >= max(picks_per_round)
    if cfg.refill.strategy == "no_refill" and cfg.draft.picks_per_round:
        max_picks = max(cfg.draft.picks_per_round)
        if cfg.draft.pack_size < max_picks:
            errors.append(
                f"pack_size={cfg.draft.pack_size} < "
                f"max(picks_per_round)={max_picks} in no_refill mode"
            )

    # Float range checks
    _check_range(errors, "agents.ai_optimality", cfg.agents.ai_optimality, 0.0, 1.0)
    _check_range(
        errors, "agents.ai_signal_weight", cfg.agents.ai_signal_weight, 0.0, 1.0
    )
    _check_range(errors, "agents.learning_rate", cfg.agents.learning_rate, 0.0, 10.0)
    _check_range(errors, "agents.ai_power_weight", cfg.agents.ai_power_weight, 0.0, 1.0)
    _check_range(errors, "agents.ai_pref_weight", cfg.agents.ai_pref_weight, 0.0, 1.0)
    _check_range(errors, "scoring.weight_power", cfg.scoring.weight_power, 0.0, 1.0)
    _check_range(
        errors, "scoring.weight_coherence", cfg.scoring.weight_coherence, 0.0, 1.0
    )
    _check_range(errors, "scoring.weight_focus", cfg.scoring.weight_focus, 0.0, 1.0)
    _check_range(
        errors, "scoring.secondary_weight", cfg.scoring.secondary_weight, 0.0, 1.0
    )
    _check_range(
        errors, "scoring.focus_threshold", cfg.scoring.focus_threshold, 0.0, 1.0
    )
    _check_range(
        errors, "scoring.focus_saturation", cfg.scoring.focus_saturation, 0.0, 1.0
    )
    _check_range(
        errors,
        "commitment.commitment_threshold",
        cfg.commitment.commitment_threshold,
        0.0,
        1.0,
    )
    _check_range(errors, "refill.fidelity", cfg.refill.fidelity, 0.0, 1.0)
    _check_range(errors, "refill.commit_bias", cfg.refill.commit_bias, 0.0, 1.0)
    _check_range(
        errors,
        "pack_generation.primary_density",
        cfg.pack_generation.primary_density,
        0.0,
        1.0,
    )
    _check_range(
        errors,
        "pack_generation.bridge_density",
        cfg.pack_generation.bridge_density,
        0.0,
        1.0,
    )
    _check_range(
        errors, "pack_generation.variance", cfg.pack_generation.variance, 0.0, 1.0
    )
    _check_range(errors, "cards.bridge_fraction", cfg.cards.bridge_fraction, 0.0, 1.0)
    _check_range(
        errors,
        "commitment.entropy_threshold",
        cfg.commitment.entropy_threshold,
        0.0,
        10.0,
    )
    _check_range(errors, "metrics.richness_gap", cfg.metrics.richness_gap, 0.0, 1.0)
    _check_range(errors, "metrics.tau", cfg.metrics.tau, 0.0, 10.0)
    _check_range(
        errors, "metrics.on_plan_threshold", cfg.metrics.on_plan_threshold, 0.0, 1.0
    )
    _check_range(
        errors,
        "metrics.splash_power_threshold",
        cfg.metrics.splash_power_threshold,
        0.0,
        1.0,
    )
    _check_range(
        errors,
        "metrics.splash_flex_threshold",
        cfg.metrics.splash_flex_threshold,
        0.0,
        1.0,
    )
    _check_range(
        errors,
        "metrics.exposure_threshold",
        cfg.metrics.exposure_threshold,
        0.0,
        1.0,
    )

    # Rarity config validation
    if cfg.rarity.enabled:
        r = cfg.rarity
        n_tiers = len(r.tiers)
        if len(r.tier_design_counts) != n_tiers:
            errors.append(
                f"rarity.tier_design_counts length {len(r.tier_design_counts)} "
                f"!= tiers length {n_tiers}"
            )
        if len(r.tier_copies) != n_tiers:
            errors.append(
                f"rarity.tier_copies length {len(r.tier_copies)} "
                f"!= tiers length {n_tiers}"
            )
        if len(r.tier_power_ranges) != n_tiers:
            errors.append(
                f"rarity.tier_power_ranges length {len(r.tier_power_ranges)} "
                f"!= tiers length {n_tiers}"
            )
        if len(r.pack_tier_slots) != n_tiers:
            errors.append(
                f"rarity.pack_tier_slots length {len(r.pack_tier_slots)} "
                f"!= tiers length {n_tiers}"
            )
        if sum(r.tier_design_counts) != cfg.cube.distinct_cards:
            errors.append(
                f"sum(rarity.tier_design_counts)={sum(r.tier_design_counts)} "
                f"!= cube.distinct_cards={cfg.cube.distinct_cards}"
            )
        if sum(r.pack_tier_slots) != cfg.draft.pack_size:
            errors.append(
                f"sum(rarity.pack_tier_slots)={sum(r.pack_tier_slots)} "
                f"!= draft.pack_size={cfg.draft.pack_size}"
            )
        for i, pr in enumerate(r.tier_power_ranges):
            if len(pr) != 2 or pr[0] < 0 or pr[1] > 1 or pr[0] > pr[1]:
                errors.append(
                    f"rarity.tier_power_ranges[{i}]={pr} invalid "
                    f"(need [low, high] in [0, 1] with low <= high)"
                )

    # Force policy requires force_archetype
    if cfg.agents.policy == "force" and cfg.agents.force_archetype is None:
        errors.append("agents.policy='force' requires agents.force_archetype to be set")

    # Positive integer checks
    if cfg.draft.seat_count < 2:
        errors.append(f"seat_count={cfg.draft.seat_count} must be >= 2")
    if cfg.draft.round_count < 1:
        errors.append(f"round_count={cfg.draft.round_count} must be >= 1")
    if cfg.draft.pack_size < 1:
        errors.append(f"pack_size={cfg.draft.pack_size} must be >= 1")
    if cfg.cube.distinct_cards < 1:
        errors.append(f"distinct_cards={cfg.cube.distinct_cards} must be >= 1")
    if cfg.cards.archetype_count < 1:
        errors.append(f"archetype_count={cfg.cards.archetype_count} must be >= 1")

    if errors:
        raise ValueError("Configuration validation failed:\n  " + "\n  ".join(errors))


def _check_range(
    errors: list[str],
    name: str,
    value: float,
    low: float,
    high: float,
) -> None:
    """Add an error if value is outside [low, high]."""
    if value < low or value > high:
        errors.append(f"{name}={value} not in [{low}, {high}]")
