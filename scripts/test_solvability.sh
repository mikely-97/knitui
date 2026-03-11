#!/usr/bin/env bash
#
# Solvability stress test for knitui board generation.
#
# Generates boards across campaign levels, endless waves, and a parameter sweep,
# then pipes them through knitui-solvcheck to verify every board has at least
# one winning pick sequence (no bonuses/blessings used).
#
# Usage:
#   cargo build --release
#   bash scripts/test_solvability.sh            # default: 50 boards per config
#   bash scripts/test_solvability.sh 200         # 200 boards per config
#
set -euo pipefail

BOARDS_PER_CONFIG="${1:-50}"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

NI="$ROOT/target/release/knitui-ni"
SOLVCHECK="$ROOT/target/release/knitui-solvcheck"

if [[ ! -x "$NI" ]] || [[ ! -x "$SOLVCHECK" ]]; then
    echo "Building release binaries..."
    cargo build --release --manifest-path "$ROOT/Cargo.toml"
fi

TOTAL_BOARDS=0
TMPFILE=$(mktemp)
trap 'rm -f "$TMPFILE"' EXIT

echo "=== Solvability Test Pipeline ==="
echo "Boards per configuration: $BOARDS_PER_CONFIG"
echo

# ── 1. Campaign levels (all tracks, all levels, zero bonuses) ────────────────

echo "--- Phase 1: Campaign levels (zero bonuses) ---"
CAMPAIGN_JSON=$("$NI" list-campaign)
TRACK_COUNT=$(echo "$CAMPAIGN_JSON" | python3 -c "import sys,json; print(len(json.load(sys.stdin)))")

for track in $(seq 0 $((TRACK_COUNT - 1))); do
    LEVEL_COUNT=$(echo "$CAMPAIGN_JSON" | python3 -c "import sys,json; print(len(json.load(sys.stdin)[$track]['levels']))")
    TRACK_NAME=$(echo "$CAMPAIGN_JSON" | python3 -c "import sys,json; print(json.load(sys.stdin)[$track]['name'])")
    echo "  Track $track ($TRACK_NAME): $LEVEL_COUNT levels x $BOARDS_PER_CONFIG boards"
    for level in $(seq 0 $((LEVEL_COUNT - 1))); do
        "$NI" --campaign --track "$track" --level "$level" \
            --scissors 0 --tweezers 0 --balloons 0 \
            batch-generate --count "$BOARDS_PER_CONFIG"
        TOTAL_BOARDS=$((TOTAL_BOARDS + BOARDS_PER_CONFIG))
    done
done > "$TMPFILE"

echo "  Generated $TOTAL_BOARDS campaign boards"
PHASE1_COUNT=$TOTAL_BOARDS

# ── 2. Endless waves 1–30 (zero bonuses) ─────────────────────────────────────

echo "--- Phase 2: Endless waves 1-30 (zero bonuses) ---"
for wave in $(seq 1 30); do
    "$NI" --endless-wave "$wave" \
        --scissors 0 --tweezers 0 --balloons 0 \
        batch-generate --count "$BOARDS_PER_CONFIG"
    TOTAL_BOARDS=$((TOTAL_BOARDS + BOARDS_PER_CONFIG))
done >> "$TMPFILE"

PHASE2_COUNT=$((TOTAL_BOARDS - PHASE1_COUNT))
echo "  Generated $PHASE2_COUNT endless boards"

# ── 3. Parameter sweep (zero bonuses) ────────────────────────────────────────

echo "--- Phase 3: Parameter sweep (zero bonuses) ---"
for h in 3 4 5 6; do
    for w in 3 4 5 6; do
        for c in 2 3 4 5 6 7 8; do
            for obs in 0 5 10 20 30; do
                "$NI" \
                    --board-height "$h" --board-width "$w" \
                    --color-number "$c" --obstacle-percentage "$obs" \
                    --conveyor-percentage 0 \
                    --scissors 0 --tweezers 0 --balloons 0 \
                    batch-generate --count "$BOARDS_PER_CONFIG"
                TOTAL_BOARDS=$((TOTAL_BOARDS + BOARDS_PER_CONFIG))
            done
        done
    done
done >> "$TMPFILE"

PHASE3_COUNT=$((TOTAL_BOARDS - PHASE1_COUNT - PHASE2_COUNT))
echo "  Generated $PHASE3_COUNT sweep boards"

# ── 4. Conveyor sweep ────────────────────────────────────────────────────────

echo "--- Phase 4: Conveyor sweep ---"
for conv in 5 10 20; do
    for h in 4 5 6; do
        "$NI" \
            --board-height "$h" --board-width "$h" \
            --color-number 4 --obstacle-percentage 5 \
            --conveyor-percentage "$conv" \
            --scissors 0 --tweezers 0 --balloons 0 \
            batch-generate --count "$BOARDS_PER_CONFIG"
        TOTAL_BOARDS=$((TOTAL_BOARDS + BOARDS_PER_CONFIG))
    done
done >> "$TMPFILE"

PHASE4_COUNT=$((TOTAL_BOARDS - PHASE1_COUNT - PHASE2_COUNT - PHASE3_COUNT))
echo "  Generated $PHASE4_COUNT conveyor boards"

# ── Run the checker ──────────────────────────────────────────────────────────

echo
echo "Total boards generated: $TOTAL_BOARDS"
echo "Running solvability checker..."
echo

if "$SOLVCHECK" < "$TMPFILE" > /dev/null; then
    echo
    echo "ALL BOARDS SOLVABLE"
    exit 0
else
    echo
    echo "SOME BOARDS UNSOLVABLE — see details above"
    exit 1
fi
