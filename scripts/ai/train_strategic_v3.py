#!/usr/bin/env python3
import argparse
import json
from pathlib import Path

import lightgbm as lgb
import numpy as np


def read_jsonl(path: Path):
    return [json.loads(line) for line in path.read_text().splitlines() if line.strip()]


def merge_features(state_features, action_features, legal_action_count):
    merged = {f"state_{k}": float(v) for k, v in state_features.items()}
    merged.update({f"action_{k}": float(v) for k, v in action_features.items()})
    merged["global_legal_action_count"] = float(legal_action_count)
    return merged


def materialize_policy(rows):
    feature_names = sorted(
        {
            *(
                key
                for row in rows
                for candidate in row["candidates"]
                for key in merge_features(
                    row["state_features"],
                    candidate["action_features"],
                    row["legal_action_count"],
                ).keys()
            )
        }
    )
    matrix = []
    labels = []
    groups = []
    for row in rows:
        if not row["candidates"]:
            continue
        groups.append(len(row["candidates"]))
        total_visits = max(
            sum(candidate["visit_count"] for candidate in row["candidates"]), 1
        )
        for candidate in row["candidates"]:
            merged = merge_features(
                row["state_features"],
                candidate["action_features"],
                row["legal_action_count"],
            )
            matrix.append([merged.get(name, 0.0) for name in feature_names])
            labels.append(int(round(candidate["visit_count"] * 30 / total_visits)))
    return feature_names, matrix, labels, groups


def materialize_value(rows):
    feature_names = sorted(
        {key for row in rows for key in row["state_features"].keys()}
    )
    matrix = [
        [row["state_features"].get(name, 0.0) for name in feature_names] for row in rows
    ]
    labels = [float(row["outcome"]) for row in rows]
    return feature_names, matrix, labels


def train_policy(rows, output_path: Path):
    feature_names, matrix, labels, groups = materialize_policy(rows)
    dataset = lgb.Dataset(
        np.asarray(matrix, dtype=float),
        label=np.asarray(labels, dtype=float),
        group=np.asarray(groups, dtype=int),
        feature_name=feature_names,
    )
    booster = lgb.train(
        {
            "objective": "lambdarank",
            "metric": "ndcg",
            "learning_rate": 0.05,
            "min_data_in_leaf": 5,
            "num_leaves": 31,
            "verbosity": -1,
        },
        dataset,
        num_boost_round=48,
    )
    output_path.write_text(
        json.dumps(
            {"feature_names": feature_names, "model": booster.dump_model()}, indent=2
        )
    )


def train_value(rows, output_path: Path):
    feature_names, matrix, labels = materialize_value(rows)
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
        num_boost_round=48,
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
        default=str(Path("/Users/dthurn/dreamtides/strategic_v3_dataset")),
    )
    parser.add_argument(
        "--output-dir",
        default=str(
            Path(
                "/Users/dthurn/dreamtides/rules_engine/src/ai_strategic/model_artifacts"
            )
        ),
    )
    args = parser.parse_args()

    dataset_dir = Path(args.dataset_dir)
    output_dir = Path(args.output_dir)
    output_dir.mkdir(parents=True, exist_ok=True)

    policy_rows = read_jsonl(dataset_dir / "policy.jsonl")
    value_rows = read_jsonl(dataset_dir / "value.jsonl")
    train_policy(policy_rows, output_dir / "strategic_v3_policy.json")
    train_value(value_rows, output_dir / "strategic_v3_value.json")
    print(f"Wrote StrategicV3 artifacts to {output_dir}")


if __name__ == "__main__":
    main()
