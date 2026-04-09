#!/usr/bin/env python3
import argparse
import json
from pathlib import Path

import lightgbm as lgb
import numpy as np

REPO_ROOT = Path(__file__).resolve().parents[2]


def read_jsonl(path: Path):
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def merge_features(state_features, action_features, legal_action_count):
    merged = {f"state_{k}": float(v) for k, v in state_features.items()}
    merged.update({f"action_{k}": float(v) for k, v in action_features.items()})
    merged["global_legal_action_count"] = float(legal_action_count)
    return merged


def flatten_rows(rows):
    if not rows:
        return []
    if "candidates" not in rows[0]:
        return rows

    flat_rows = []
    for row in rows:
        total_visits = max(
            sum(candidate["visit_count"] for candidate in row["candidates"]), 1
        )
        for candidate in row["candidates"]:
            flat_rows.append(
                {
                    "action": candidate.get("action", candidate["action_short"]),
                    "action_features": candidate["action_features"],
                    "action_short": candidate["action_short"],
                    "avg_reward": float(candidate["avg_reward"]),
                    "chosen": bool(candidate["chosen"]),
                    "legal_action_count": int(row["legal_action_count"]),
                    "player": row["player"],
                    "seed": int(row["seed"]),
                    "state_features": row["state_features"],
                    "turn_id": int(row["turn_id"]),
                    "visit_count": int(candidate["visit_count"]),
                    "visit_fraction": float(candidate["visit_count"])
                    / float(total_visits),
                }
            )
    return flat_rows


def materialize_policy(rows):
    feature_names = sorted(
        {
            *(
                key
                for row in rows
                for key in merge_features(
                    row["state_features"],
                    row["action_features"],
                    row["legal_action_count"],
                ).keys()
            )
        }
    )
    matrix = []
    labels = []
    for row in rows:
        merged = merge_features(
            row["state_features"], row["action_features"], row["legal_action_count"]
        )
        matrix.append([merged.get(name, 0.0) for name in feature_names])
        labels.append(float(row["visit_fraction"]))
    return feature_names, matrix, labels


def train_policy(rows, output_path: Path):
    feature_names, matrix, labels = materialize_policy(rows)
    dataset = lgb.Dataset(
        np.asarray(matrix, dtype=float),
        label=np.asarray(labels, dtype=float),
        feature_name=feature_names,
    )
    booster = lgb.train(
        {
            "objective": "regression",
            "metric": "l2",
            "learning_rate": 0.05,
            "min_data_in_leaf": 5,
            "num_leaves": 31,
            "verbosity": -1,
        },
        dataset,
        num_boost_round=64,
    )
    output_path.write_text(
        json.dumps(
            {"feature_names": feature_names, "model": booster.dump_model()}, indent=2
        )
    )


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--dataset-dir",
        default=str(Path("/Users/dthurn/strategic_v3_training/hour1_core11_t20")),
    )
    parser.add_argument(
        "--output-dir",
        default=str(REPO_ROOT / "rules_engine/src/ai_uct/model_artifacts"),
    )
    args = parser.parse_args()

    dataset_dir = Path(args.dataset_dir)
    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    policy_rows = flatten_rows(read_jsonl(dataset_dir / "policy.jsonl"))
    train_policy(policy_rows, output_dir / "monte_carlo_hybrid_v1_policy.json")
    print(f"Wrote MonteCarloHybridV1 artifacts to {output_dir}")


if __name__ == "__main__":
    main()
