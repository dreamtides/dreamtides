#!/usr/bin/env python3
"""Rebalance all 503 cards across rarity, resonance, and archetypes."""

import tomllib
import re
import random
import copy
import sys
from pathlib import Path
from collections import Counter

ROOT = Path(__file__).resolve().parent.parent.parent
RENDERED_CARDS = ROOT / "scripts/quest_simulator/data/rendered-cards.toml"
CARDS_TOML = ROOT / "rules_engine/tabula/cards.toml"
CLIENT_META = ROOT / "client/Assets/StreamingAssets/Tabula/card-metadata.toml"
ENGINE_META = ROOT / "rules_engine/tabula/card-metadata.toml"

# Archetypes that matter for balancing
ARCHETYPES = [
    "flash",
    "awaken",
    "flicker",
    "ignite",
    "shatter",
    "endure",
    "submerge",
    "surge",
]

# Valid dual resonance pairs (archetype alliances)
VALID_DUAL_PAIRS = [
    ("Stone", "Tide"),
    ("Stone", "Flame"),
    ("Flame", "Thunder"),
    ("Thunder", "Tide"),
]

# Archetype-to-resonance affinities
ARCHETYPE_RESONANCE = {
    "endure": "Stone",
    "submerge": "Stone",
    "awaken": "Tide",
    "flicker": "Tide",
    "ignite": "Flame",
    "flash": "Flame",
    "shatter": "Thunder",
    "surge": "Thunder",
}

RESONANCE_ARCHETYPES = {}
for arch, res in ARCHETYPE_RESONANCE.items():
    RESONANCE_ARCHETYPES.setdefault(res, []).append(arch)


def load_toml(path):
    with open(path, "rb") as f:
        return tomllib.load(f)


def load_raw(path):
    with open(path, "r") as f:
        return f.read()


# ============================================================
# TOML WRITING HELPERS (preserve structure, line-level edits)
# ============================================================


def write_toml_cards(path, cards_data, key="cards"):
    """Write a TOML file with [[cards]] or [[card-metadata]] entries."""
    lines = []
    for card in cards_data:
        lines.append(f"[[{key}]]")
        for k, v in card.items():
            lines.append(f"{format_key(k)} = {format_value(v)}")
        lines.append("")
    with open(path, "w") as f:
        f.write("\n".join(lines) + "\n")


def format_key(k):
    if " " in k or "-" not in k and k.isidentifier():
        # Keys with spaces need quoting
        pass
    # Check if key needs quoting
    if any(c in k for c in " "):
        return f'"{k}"'
    return k


def format_value(v):
    if isinstance(v, str):
        if "\n" in v:
            return f'"""{v}"""'
        return f'"{v}"'
    elif isinstance(v, bool):
        return "true" if v else "false"
    elif isinstance(v, int):
        return str(v)
    elif isinstance(v, float):
        # Format without unnecessary trailing zeros but keep at least one decimal
        if v == int(v):
            return str(int(v))
        return str(v)
    elif isinstance(v, list):
        return "[" + ", ".join(format_value(x) for x in v) + "]"
    else:
        return str(v)


# ============================================================
# LINE-LEVEL EDITING for rendered-cards.toml and cards.toml
# These files have specific formatting we want to preserve.
# ============================================================


def parse_card_blocks(text):
    """Parse a TOML file into blocks, each block is one [[cards]] entry.
    Returns list of (start_line, end_line, card_dict, raw_lines)."""
    lines = text.split("\n")
    blocks = []
    current_start = None
    current_lines = []

    for i, line in enumerate(lines):
        if line.strip() == "[[cards]]":
            if current_start is not None:
                blocks.append((current_start, i - 1, current_lines))
            current_start = i
            current_lines = [line]
        elif current_start is not None:
            current_lines.append(line)

    if current_start is not None:
        blocks.append((current_start, len(lines) - 1, current_lines))

    return blocks


def get_field_from_block(block_lines, field):
    """Extract a field value from a block's raw lines."""
    for line in block_lines:
        # Match field = value
        key = f"{field} = " if " " not in field else f'"{field}" = '
        if line.strip().startswith(key):
            val_str = line.strip()[len(key) :]
            # Parse simple values
            if val_str.startswith("["):
                # Parse list
                inner = val_str[1:-1].strip()
                if not inner:
                    return []
                return [x.strip().strip('"') for x in inner.split(",")]
            elif val_str.startswith('"'):
                return val_str.strip('"')
            elif val_str == "true":
                return True
            elif val_str == "false":
                return False
            else:
                try:
                    return int(val_str)
                except ValueError:
                    try:
                        return float(val_str)
                    except ValueError:
                        return val_str
    return None


def set_field_in_block(block_lines, field, value):
    """Set or add a field in a block's raw lines. Returns modified lines."""
    key_prefix = f"{field} = " if " " not in field else f'"{field}" = '
    formatted = f"{key_prefix}{format_value(value)}"

    # Use the unquoted form for the key match
    key_match = field if " " not in field else f'"{field}"'

    for i, line in enumerate(block_lines):
        stripped = line.strip()
        if stripped.startswith(f"{key_match} = "):
            block_lines[i] = formatted
            return block_lines

    # Field not found - add it before the last empty line or at end
    # Find the right place to insert (after the last non-empty line)
    insert_pos = len(block_lines)
    for i in range(len(block_lines) - 1, 0, -1):
        if block_lines[i].strip():
            insert_pos = i + 1
            break
    block_lines.insert(insert_pos, formatted)
    return block_lines


def remove_field_from_block(block_lines, field):
    """Remove a field from a block's raw lines."""
    key_match = field if " " not in field else f'"{field}"'
    block_lines[:] = [
        line for line in block_lines if not line.strip().startswith(f"{key_match} = ")
    ]
    return block_lines


def rebuild_file_from_blocks(blocks):
    """Rebuild file content from modified blocks."""
    parts = []
    for _, _, lines in blocks:
        parts.append("\n".join(lines))
    return "\n".join(parts)


# ============================================================
# PHASE 1: Sync resonances from cards.toml to rendered-cards.toml
# ============================================================


def phase1_sync_resonances():
    """Copy resonance from cards.toml to rendered-cards.toml for old cards."""
    print("=== Phase 1: Sync old card resonances ===")

    cards_data = load_toml(CARDS_TOML)
    # Build id -> resonance map from cards.toml
    id_to_resonance = {}
    for c in cards_data["cards"]:
        id_to_resonance[c["id"]] = c.get("resonance", [])

    # Parse rendered-cards.toml line by line
    raw = load_raw(RENDERED_CARDS)
    blocks = parse_card_blocks(raw)

    changed = 0
    for start, end, lines in blocks:
        card_id = get_field_from_block(lines, "id")
        card_num = get_field_from_block(lines, "card-number")

        if card_id and card_id in id_to_resonance:
            new_res = id_to_resonance[card_id]
            old_res = get_field_from_block(lines, "resonance")

            if old_res is None and not new_res:
                continue  # Both neutral, no change needed

            if new_res:
                set_field_in_block(lines, "resonance", new_res)
            else:
                # Remove resonance field (neutral card)
                remove_field_from_block(lines, "resonance")

            if old_res != new_res:
                changed += 1

    result = rebuild_file_from_blocks(blocks)
    with open(RENDERED_CARDS, "w") as f:
        f.write(result)

    print(f"  Updated {changed} cards' resonances in rendered-cards.toml")
    return changed


# ============================================================
# PHASE 2: Whole-pool resonance balancing
# ============================================================


def count_resonances(cards):
    """Count per-resonance appearances, neutral, single, dual."""
    res_count = Counter()
    neutral = 0
    single = 0
    dual = 0
    for c in cards:
        r = c.get("resonance", [])
        if not r:
            neutral += 1
        elif len(r) == 1:
            single += 1
            res_count[r[0]] += 1
        elif len(r) == 2:
            dual += 1
            for x in r:
                res_count[x] += 1
    return res_count, neutral, single, dual


def get_primary_archetype(meta_entry):
    """Get the archetype with highest score."""
    best = None
    best_score = -1
    for a in ARCHETYPES:
        s = meta_entry.get(a, 0)
        if s > best_score:
            best_score = s
            best = a
    return best, best_score


def get_archetype_resonance_fit(meta_entry, resonance):
    """Score how well a card's archetypes fit a resonance."""
    archs = RESONANCE_ARCHETYPES.get(resonance, [])
    return sum(meta_entry.get(a, 0) for a in archs)


def phase2_resonance_balancing():
    """Balance resonances across the full card pool."""
    print("\n=== Phase 2: Whole-pool resonance balancing ===")

    rendered = load_toml(RENDERED_CARDS)
    meta_data = load_toml(CLIENT_META)

    # Build metadata lookup
    meta_by_id = {m["card-id"]: m for m in meta_data["card-metadata"]}

    cards = rendered["cards"]

    # Exclude Legendary/Special
    rebalanceable = [
        c for c in cards if c.get("rarity") not in ("Legendary", "Special")
    ]

    res_count, neutral_count, single_count, dual_count = count_resonances(rebalanceable)
    print(
        f"  Before: Stone={res_count['Stone']}, Tide={res_count['Tide']}, "
        f"Flame={res_count['Flame']}, Thunder={res_count['Thunder']}"
    )
    print(f"  Neutral={neutral_count}, Single={single_count}, Dual={dual_count}")

    # Target: ~75 dual, each resonance ~130 appearances
    TARGET_DUAL = 75
    TARGET_PER_RES = 130

    # Step 2a: Promote singles to duals
    new_duals_needed = TARGET_DUAL - dual_count
    print(f"  Need {new_duals_needed} more dual cards")

    # Find single-resonance candidates for dual promotion
    candidates = []
    for c in rebalanceable:
        r = c.get("resonance", [])
        if len(r) != 1:
            continue
        current_res = r[0]
        meta = meta_by_id.get(c["id"])
        if not meta:
            continue

        # Find the best second resonance
        for pair in VALID_DUAL_PAIRS:
            if current_res in pair:
                second_res = pair[0] if pair[1] == current_res else pair[1]
                fit = get_archetype_resonance_fit(meta, second_res)
                if fit >= 0.4:  # Reasonable affinity threshold
                    candidates.append((c, current_res, second_res, fit))

    # Sort by fit score (higher = better match for dual)
    candidates.sort(key=lambda x: -x[3])

    # Prioritize: pull from over-represented, push to under-represented
    def priority_score(current, second):
        over = max(0, res_count[current] - TARGET_PER_RES)
        under = max(0, TARGET_PER_RES - res_count.get(second, 0))
        return over + under

    candidates.sort(key=lambda x: (-priority_score(x[1], x[2]), -x[3]))

    promoted = 0
    promoted_ids = set()
    for c, current_res, second_res, fit in candidates:
        if promoted >= new_duals_needed:
            break
        if c["id"] in promoted_ids:
            continue

        c["resonance"] = sorted([current_res, second_res])
        promoted_ids.add(c["id"])
        res_count[second_res] += 1
        promoted += 1

    print(f"  Promoted {promoted} cards to dual resonance")

    # Step 2b: Rebalance single resonances
    res_count2, _, _, _ = count_resonances(rebalanceable)
    print(
        f"  After dual promotion: Stone={res_count2['Stone']}, Tide={res_count2['Tide']}, "
        f"Flame={res_count2['Flame']}, Thunder={res_count2['Thunder']}"
    )

    # Move cards from over-represented to under-represented
    over_res = [
        r
        for r in ["Stone", "Tide", "Flame", "Thunder"]
        if res_count2[r] > TARGET_PER_RES + 5
    ]
    under_res = [
        r
        for r in ["Stone", "Tide", "Flame", "Thunder"]
        if res_count2[r] < TARGET_PER_RES - 5
    ]

    moves = 0
    for from_res in over_res:
        excess = res_count2[from_res] - TARGET_PER_RES
        if excess <= 0:
            continue

        # Find single-res cards in from_res that could fit an under_res
        move_candidates = []
        for c in rebalanceable:
            r = c.get("resonance", [])
            if len(r) != 1 or r[0] != from_res:
                continue
            if c["id"] in promoted_ids:
                continue

            meta = meta_by_id.get(c["id"])
            if not meta:
                continue

            for to_res in under_res:
                if res_count2[to_res] >= TARGET_PER_RES:
                    continue
                fit_new = get_archetype_resonance_fit(meta, to_res)
                fit_old = get_archetype_resonance_fit(meta, from_res)
                if fit_new >= fit_old * 0.7:  # Reasonable fit for new resonance
                    move_candidates.append((c, to_res, fit_new))

        move_candidates.sort(key=lambda x: -x[2])

        for c, to_res, fit in move_candidates:
            if res_count2[from_res] <= TARGET_PER_RES:
                break
            if res_count2[to_res] >= TARGET_PER_RES:
                continue

            c["resonance"] = [to_res]
            res_count2[from_res] -= 1
            res_count2[to_res] += 1
            moves += 1

    print(f"  Moved {moves} cards between resonances")

    res_final, n_final, s_final, d_final = count_resonances(rebalanceable)
    print(
        f"  Final: Stone={res_final['Stone']}, Tide={res_final['Tide']}, "
        f"Flame={res_final['Flame']}, Thunder={res_final['Thunder']}"
    )
    print(f"  Neutral={n_final}, Single={s_final}, Dual={d_final}")

    # Write changes back to rendered-cards.toml
    apply_resonance_changes(cards)

    # Also update cards.toml for old cards
    apply_resonance_to_cards_toml(cards)

    return cards


def apply_resonance_changes(cards):
    """Apply resonance changes to rendered-cards.toml using line-level edits."""
    raw = load_raw(RENDERED_CARDS)
    blocks = parse_card_blocks(raw)

    # Build id -> card map from modified data
    id_to_card = {c["id"]: c for c in cards}

    for start, end, lines in blocks:
        card_id = get_field_from_block(lines, "id")
        if card_id not in id_to_card:
            continue

        card = id_to_card[card_id]
        new_res = card.get("resonance", [])
        old_res = get_field_from_block(lines, "resonance")

        if new_res:
            set_field_in_block(lines, "resonance", new_res)
        elif old_res is not None:
            remove_field_from_block(lines, "resonance")

    result = rebuild_file_from_blocks(blocks)
    with open(RENDERED_CARDS, "w") as f:
        f.write(result)


def apply_resonance_to_cards_toml(rendered_cards):
    """Update cards.toml resonance for old cards based on rendered data."""
    raw = load_raw(CARDS_TOML)
    blocks = parse_card_blocks(raw)

    # Build id -> rendered card map (only old cards)
    id_to_rendered = {}
    for c in rendered_cards:
        if c.get("card-number", 999) <= 222:
            id_to_rendered[c["id"]] = c

    changed = 0
    for start, end, lines in blocks:
        card_id = get_field_from_block(lines, "id")
        if card_id not in id_to_rendered:
            continue

        rc = id_to_rendered[card_id]
        new_res = rc.get("resonance", [])
        old_res = get_field_from_block(lines, "resonance")

        if new_res != (old_res or []):
            if new_res:
                set_field_in_block(lines, "resonance", new_res)
            elif old_res is not None:
                remove_field_from_block(lines, "resonance")
            changed += 1

    result = rebuild_file_from_blocks(blocks)
    with open(CARDS_TOML, "w") as f:
        f.write(result)

    print(f"  Updated {changed} cards in cards.toml")


# ============================================================
# PHASE 3: Rarity rebalancing
# ============================================================


def phase3_rarity_rebalancing():
    """Rebalance rarities to 166 C / 166 U / 165 R."""
    print("\n=== Phase 3: Rarity rebalancing ===")

    rendered = load_toml(RENDERED_CARDS)
    cards = rendered["cards"]

    rebalanceable = [
        c for c in cards if c.get("rarity") not in ("Legendary", "Special")
    ]

    rarity_count = Counter(c["rarity"] for c in rebalanceable)
    print(
        f"  Before: C={rarity_count['Common']}, U={rarity_count['Uncommon']}, R={rarity_count['Rare']}"
    )

    # Target: 166 C, 166 U, 165 R
    # Currently: 171 C, 181 U, 145 R
    # Need: -5 C, -15 U, +20 R

    # Score cards by complexity (text length as proxy)
    def complexity_score(card):
        text = card.get("rendered text", "")
        score = len(text)
        # Bonus for triggers/keywords
        keywords = [
            "Materialized",
            "Judgment",
            "Foresee",
            "Kindle",
            "Prevent",
            "Dissolve",
            "Reclaim",
            "Banish",
            "When",
            "each turn",
            "for each",
            "instead",
        ]
        for kw in keywords:
            if kw.lower() in text.lower():
                score += 30
        return score

    # Find Uncommon cards that should be Rare (most complex Uncommons)
    uncommons = [c for c in rebalanceable if c["rarity"] == "Uncommon"]
    uncommons.sort(key=complexity_score, reverse=True)

    # Promote top 15 Uncommons to Rare
    u_to_r = uncommons[:15]
    for c in u_to_r:
        c["rarity"] = "Rare"

    # Find Common cards that should be Rare (most complex Commons)
    commons = [c for c in rebalanceable if c["rarity"] == "Common"]
    commons.sort(key=complexity_score, reverse=True)

    # Promote top 5 Commons to Rare
    c_to_r = commons[:5]
    for c in c_to_r:
        c["rarity"] = "Rare"

    # Verify
    rarity_count2 = Counter(c["rarity"] for c in rebalanceable)
    print(
        f"  After: C={rarity_count2['Common']}, U={rarity_count2['Uncommon']}, R={rarity_count2['Rare']}"
    )

    # Apply changes
    apply_rarity_changes(cards)

    # Also update cards.toml for old cards
    apply_rarity_to_cards_toml(cards)

    promoted_ids = set(c["id"] for c in u_to_r + c_to_r)
    print(f"  Promoted {len(u_to_r)} Uncommon→Rare, {len(c_to_r)} Common→Rare")
    return promoted_ids


def apply_rarity_changes(cards):
    """Apply rarity changes to rendered-cards.toml."""
    raw = load_raw(RENDERED_CARDS)
    blocks = parse_card_blocks(raw)

    id_to_card = {c["id"]: c for c in cards}

    for start, end, lines in blocks:
        card_id = get_field_from_block(lines, "id")
        if card_id not in id_to_card:
            continue
        card = id_to_card[card_id]
        set_field_in_block(lines, "rarity", card["rarity"])

    result = rebuild_file_from_blocks(blocks)
    with open(RENDERED_CARDS, "w") as f:
        f.write(result)


def apply_rarity_to_cards_toml(rendered_cards):
    """Update rarity in cards.toml for old cards."""
    raw = load_raw(CARDS_TOML)
    blocks = parse_card_blocks(raw)

    id_to_rendered = {
        c["id"]: c for c in rendered_cards if c.get("card-number", 999) <= 222
    }

    changed = 0
    for start, end, lines in blocks:
        card_id = get_field_from_block(lines, "id")
        if card_id not in id_to_rendered:
            continue

        rc = id_to_rendered[card_id]
        old_rarity = get_field_from_block(lines, "rarity")
        if old_rarity != rc["rarity"]:
            set_field_in_block(lines, "rarity", rc["rarity"])
            changed += 1

    result = rebuild_file_from_blocks(blocks)
    with open(CARDS_TOML, "w") as f:
        f.write(result)

    print(f"  Updated {changed} rarities in cards.toml")


# ============================================================
# PHASE 4: Archetype score adjustments
# ============================================================


def phase4_archetype_adjustments():
    """Adjust archetype scores so each archetype has 79-89 primaries."""
    print("\n=== Phase 4: Archetype score adjustments ===")

    meta = load_toml(CLIENT_META)
    entries = meta["card-metadata"]

    # Current counts
    counts = {a: sum(1 for m in entries if m.get(a, 0) >= 0.6) for a in ARCHETYPES}
    print(f"  Before: {counts}")

    TARGET_MIN = 79
    TARGET_MAX = 89

    # Over-represented: reduce
    for arch in ARCHETYPES:
        if counts[arch] <= TARGET_MAX:
            continue

        excess = counts[arch] - TARGET_MAX
        # Find cards where this archetype is 0.60-0.65 and not the primary
        candidates = []
        for m in entries:
            score = m.get(arch, 0)
            if 0.60 <= score <= 0.65:
                primary, primary_score = get_primary_archetype(m)
                if primary != arch:
                    candidates.append((m, score))

        candidates.sort(key=lambda x: x[1])  # Lowest first

        reduced = 0
        for m, score in candidates:
            if reduced >= excess:
                break
            m[arch] = 0.55
            reduced += 1
            counts[arch] -= 1

        print(f"  Reduced {arch}: {reduced} cards lowered to 0.55")

    # Under-represented: increase
    for arch in ARCHETYPES:
        if counts[arch] >= TARGET_MIN:
            continue

        deficit = TARGET_MIN - counts[arch]
        target_res = ARCHETYPE_RESONANCE.get(arch)

        # Load rendered cards to check resonance alignment
        rendered = load_toml(RENDERED_CARDS)
        id_to_res = {c["id"]: c.get("resonance", []) for c in rendered["cards"]}

        candidates = []
        for m in entries:
            score = m.get(arch, 0)
            if 0.50 <= score < 0.60:
                card_res = id_to_res.get(m["card-id"], [])
                res_match = target_res in card_res if target_res else False
                candidates.append((m, score, res_match))

        # Prefer resonance-aligned cards
        candidates.sort(key=lambda x: (-int(x[2]), -x[1]))

        increased = 0
        for m, score, res_match in candidates:
            if increased >= deficit:
                break
            m[arch] = 0.6
            increased += 1
            counts[arch] += 1

        print(f"  Increased {arch}: {increased} cards raised to 0.60")

    print(f"  After: {counts}")

    # Write to both metadata files
    write_metadata(entries)

    return counts


def write_metadata(entries):
    """Write metadata to both card-metadata.toml files."""
    # Build the output preserving field order
    lines = []
    for m in entries:
        lines.append("[[card-metadata]]")
        lines.append(f'card-id = "{m["card-id"]}"')
        for field in [
            "flash",
            "awaken",
            "flicker",
            "ignite",
            "shatter",
            "endure",
            "submerge",
            "surge",
            "power",
            "commit",
            "flex",
        ]:
            val = m.get(field, 0)
            if isinstance(val, float):
                # Format cleanly
                if val == int(val):
                    lines.append(f"{field} = {int(val)}")
                else:
                    lines.append(f"{field} = {val}")
            else:
                lines.append(f"{field} = {val}")
        lines.append("")

    content = "\n".join(lines) + "\n"
    with open(CLIENT_META, "w") as f:
        f.write(content)
    with open(ENGINE_META, "w") as f:
        f.write(content)

    print(f"  Wrote metadata to both files ({len(entries)} entries)")


# ============================================================
# PHASE 5: Final validation
# ============================================================


def phase5_validate():
    """Run comprehensive validation checks."""
    print("\n=== Phase 5: Final validation ===")

    rendered = load_toml(RENDERED_CARDS)
    cards_toml = load_toml(CARDS_TOML)
    client_meta = load_toml(CLIENT_META)
    engine_meta = load_toml(ENGINE_META)

    errors = []
    warnings = []

    cards = rendered["cards"]

    # 1. Card counts
    if len(cards) != 503:
        errors.append(f"Expected 503 cards in rendered-cards.toml, got {len(cards)}")
    if len(client_meta["card-metadata"]) != 503:
        errors.append(
            f"Expected 503 metadata entries in client, got {len(client_meta['card-metadata'])}"
        )
    if len(engine_meta["card-metadata"]) != 503:
        errors.append(
            f"Expected 503 metadata entries in engine, got {len(engine_meta['card-metadata'])}"
        )
    print(
        f"  Card counts: rendered={len(cards)}, client-meta={len(client_meta['card-metadata'])}, engine-meta={len(engine_meta['card-metadata'])}"
    )

    # 2. UUID matching
    rendered_ids = {c["id"] for c in cards}
    client_ids = {m["card-id"] for m in client_meta["card-metadata"]}
    engine_ids = {m["card-id"] for m in engine_meta["card-metadata"]}

    if rendered_ids != client_ids:
        missing = rendered_ids - client_ids
        extra = client_ids - rendered_ids
        if missing:
            errors.append(f"Cards missing from client metadata: {len(missing)}")
        if extra:
            errors.append(f"Extra entries in client metadata: {len(extra)}")

    if client_ids != engine_ids:
        errors.append("Client and engine metadata IDs don't match")
    print(
        f"  UUID match: rendered↔client={'OK' if rendered_ids == client_ids else 'MISMATCH'}, "
        f"client↔engine={'OK' if client_ids == engine_ids else 'MISMATCH'}"
    )

    # 3. No legacy resonance names
    legacy_count = 0
    for c in cards:
        for r in c.get("resonance", []):
            if r in ("Ember", "Ruin", "Zephyr"):
                legacy_count += 1
                errors.append(
                    f"Legacy resonance '{r}' found on card {c.get('card-number')}: {c['name']}"
                )
    if legacy_count == 0:
        print("  No legacy resonance names found: OK")
    else:
        print(f"  LEGACY RESONANCES FOUND: {legacy_count}")

    # 4. Rarity distribution
    rarity_count = Counter(c["rarity"] for c in cards)
    print(
        f"  Rarity: C={rarity_count.get('Common', 0)}, U={rarity_count.get('Uncommon', 0)}, "
        f"R={rarity_count.get('Rare', 0)}, L={rarity_count.get('Legendary', 0)}, "
        f"S={rarity_count.get('Special', 0)}"
    )

    non_ls = sum(
        v for k, v in rarity_count.items() if k not in ("Legendary", "Special")
    )
    if non_ls != 497:
        warnings.append(f"Expected 497 non-L/S cards, got {non_ls}")

    expected_rarity = {
        "Common": 166,
        "Uncommon": 166,
        "Rare": 165,
        "Legendary": 4,
        "Special": 2,
    }
    for r, expected in expected_rarity.items():
        actual = rarity_count.get(r, 0)
        if actual != expected:
            warnings.append(f"Rarity {r}: expected {expected}, got {actual}")

    # 5. Resonance distribution
    rebalanceable = [
        c for c in cards if c.get("rarity") not in ("Legendary", "Special")
    ]
    res_count, neutral, single, dual = count_resonances(rebalanceable)
    print(
        f"  Resonance: Stone={res_count.get('Stone', 0)}, Tide={res_count.get('Tide', 0)}, "
        f"Flame={res_count.get('Flame', 0)}, Thunder={res_count.get('Thunder', 0)}"
    )
    print(f"  Types: neutral={neutral}, single={single}, dual={dual}")

    for r in ["Stone", "Tide", "Flame", "Thunder"]:
        if not (120 <= res_count.get(r, 0) <= 140):
            warnings.append(
                f"Resonance {r}={res_count.get(r, 0)} outside 120-140 range"
            )

    if not (45 <= neutral <= 55):
        warnings.append(f"Neutral count {neutral} outside 45-55 range")
    if not (65 <= dual <= 85):
        warnings.append(f"Dual count {dual} outside 65-85 range")

    # 6. Archetype distribution
    arch_counts = {
        a: sum(1 for m in client_meta["card-metadata"] if m.get(a, 0) >= 0.6)
        for a in ARCHETYPES
    }
    print(f"  Archetypes (>=0.6): {arch_counts}")
    for a, c in arch_counts.items():
        if not (79 <= c <= 89):
            warnings.append(f"Archetype {a}={c} outside 79-89 range")

    # 7. Metadata files identical
    with open(CLIENT_META, "r") as f:
        client_raw = f.read()
    with open(ENGINE_META, "r") as f:
        engine_raw = f.read()
    if client_raw == engine_raw:
        print("  Metadata files identical: OK")
    else:
        errors.append("Metadata files are not identical!")

    # 8. cards.toml rarity matches rendered-cards.toml for 1-222
    cards_toml_by_id = {c["id"]: c for c in cards_toml["cards"]}
    rarity_mismatches = 0
    for c in cards:
        if c.get("card-number", 999) > 222:
            continue
        ct = cards_toml_by_id.get(c["id"])
        if ct and ct.get("rarity") != c.get("rarity"):
            rarity_mismatches += 1
            errors.append(
                f"Rarity mismatch for card {c['card-number']} ({c['name']}): "
                f"rendered={c.get('rarity')} vs cards.toml={ct.get('rarity')}"
            )

    # Also check resonance matches for old cards
    res_mismatches = 0
    for c in cards:
        if c.get("card-number", 999) > 222:
            continue
        ct = cards_toml_by_id.get(c["id"])
        if ct:
            r_rendered = c.get("resonance", [])
            r_cards = ct.get("resonance", [])
            if sorted(r_rendered) != sorted(r_cards):
                res_mismatches += 1
                errors.append(
                    f"Resonance mismatch for card {c['card-number']} ({c['name']}): "
                    f"rendered={r_rendered} vs cards.toml={r_cards}"
                )

    print(
        f"  Rarity sync (1-222): {'OK' if rarity_mismatches == 0 else f'{rarity_mismatches} mismatches'}"
    )
    print(
        f"  Resonance sync (1-222): {'OK' if res_mismatches == 0 else f'{res_mismatches} mismatches'}"
    )

    # Summary
    print(f"\n{'='*50}")
    if errors:
        print(f"ERRORS ({len(errors)}):")
        for e in errors[:20]:
            print(f"  ✗ {e}")
    if warnings:
        print(f"WARNINGS ({len(warnings)}):")
        for w in warnings[:20]:
            print(f"  ! {w}")
    if not errors and not warnings:
        print("ALL CHECKS PASSED!")
    elif not errors:
        print(f"No errors, {len(warnings)} warnings (minor deviations from targets)")
    else:
        print(f"{len(errors)} errors, {len(warnings)} warnings")

    return len(errors) == 0


# ============================================================
# MAIN
# ============================================================


def main():
    random.seed(42)  # Reproducible results

    print("Rebalancing 503 cards across rarity, resonance, and archetypes")
    print(f"Root: {ROOT}\n")

    phase1_sync_resonances()
    phase2_resonance_balancing()
    phase3_rarity_rebalancing()
    phase4_archetype_adjustments()
    success = phase5_validate()

    sys.exit(0 if success else 1)


if __name__ == "__main__":
    main()
