#!/bin/bash
# Copyright (c) ÆthelingTeam and affiliates.
#
# This source code is licensed under the AGPL-3.0 license found in the
# LICENSE file in the root directory of this source tree.

# Starts config-server, game-server, and muip-server under tini.
# game-server [muip]  -> host/port = GM bridge TCP listener
# muip-server [muip]  -> host/port = HTTP admin panel; gm_host/gm_port = GM bridge client
# Separate configs are generated to avoid port conflicts between the two.

set -euo pipefail

USER_CONFIG="${CONFIG_FILE:-/app/Config.toml}"
DEFAULT_CONFIG="/app/config.default.toml"
GAME_CONFIG="/tmp/game-config.toml"
MUIP_CONFIG="/tmp/muip-config.toml"

# Ensure saves directory exists
mkdir -p /app/saves

# Generate Config.toml from defaults if the user did not mount one
if [ ! -f "$USER_CONFIG" ]; then
    echo "[entrypoint] No Config.toml found, generating from defaults..."
    cp "$DEFAULT_CONFIG" "$USER_CONFIG"
fi

# Parse a value from a flat TOML [section] using python3, falling back to awk
parse_toml_value() {
    local file="$1" section="$2" key="$3" default="$4"
    python3 -c "
import tomllib
try:
    with open('$file', 'rb') as f:
        d = tomllib.load(f)
    print(d.get('$section', {}).get('$key', '$default'))
except Exception:
    print('$default')
" 2>/dev/null || echo "$default"
}

if ! command -v python3 &>/dev/null; then
    parse_toml_value() {
        local file="$1" section="$2" key="$3" default="$4"
        awk -v sec="[$2]" -v k="$3" -v def="$4" '
            $0 == sec          { in_section = 1; next }
            /^\[/ && $0 != sec { in_section = 0; next }
            in_section && $1 == k {
                found = 1
                gsub(/.*= *"?/, ""); gsub(/"? *$/, ""); print; exit
            }
            END { if (!found) print def }
        ' "$file" 2>/dev/null
    }
fi

# Read settings from user config
GAME_HOST=$(parse_toml_value "$USER_CONFIG" server  host    "0.0.0.0")
GAME_PORT=$(parse_toml_value "$USER_CONFIG" server  port    "1337")
ASSETS_PATH=$(parse_toml_value "$USER_CONFIG" assets path   "assets")
GM_BRIDGE_HOST=$(parse_toml_value "$USER_CONFIG" muip_gm host "127.0.0.1")
GM_BRIDGE_PORT=$(parse_toml_value "$USER_CONFIG" muip_gm port "2338")
GM_ENABLED=$(parse_toml_value "$USER_CONFIG" muip_gm enabled "true")
MUIP_PORT=$(parse_toml_value "$USER_CONFIG" muip    port    "8080")
MUIP_TOKEN=$(parse_toml_value "$USER_CONFIG" muip   token   "change-me")

# Reject values that would break TOML string literals
for var_name in GAME_HOST ASSETS_PATH GM_BRIDGE_HOST MUIP_TOKEN; do
    val="${!var_name}"
    if [[ "$val" == *'"'* || "$val" == *$'\n'* ]]; then
        echo "[entrypoint] ERROR: $var_name contains invalid characters for TOML" >&2
        exit 1
    fi
done

# Write game-server config ([muip] section = GM bridge listener)
cat > "$GAME_CONFIG" << GAMECFG
[server]
host = "${GAME_HOST}"
port = ${GAME_PORT}

[assets]
path = "${ASSETS_PATH}"

[world_state]
role_level = 1
role_exp = 0
last_scene = "map01_lv001"
pos_x = 469.0
pos_y = 107.11
pos_z = 217.83
rot_x = 0.0
rot_y = 60.00
rot_z = 0.0

[default_team]
team = [
"chr_0013_aglina",
"chr_0004_pelica",
"chr_0005_chen",
"chr_0006_wolfgd",
]

[muip]
host = "${GM_BRIDGE_HOST}"
port = ${GM_BRIDGE_PORT}
enabled = ${GM_ENABLED}
GAMECFG

# Write muip-server config ([muip] section = HTTP admin panel + GM bridge client)
cat > "$MUIP_CONFIG" << MUIPCFG
[muip]
host = "0.0.0.0"
port = ${MUIP_PORT}
token = "${MUIP_TOKEN}"
gm_host = "${GM_BRIDGE_HOST}"
gm_port = ${GM_BRIDGE_PORT}
MUIPCFG

echo "[entrypoint] Generated game-server config (GM bridge: ${GM_BRIDGE_HOST}:${GM_BRIDGE_PORT})"
echo "[entrypoint] Generated muip-server config  (admin panel: 0.0.0.0:${MUIP_PORT})"

# Set up PID tracking directory
PIDDIR="/tmp/perlica-pids"
mkdir -p "$PIDDIR"
rm -f "$PIDDIR"/*.pid

echo "============================================"
echo "  Perlica-RS Game Server, Starting"
echo "============================================"
echo ""

_SHUTTING_DOWN=false

# Send SIGTERM to all children, wait up to 10s, then SIGKILL any stragglers
cleanup() {
    # Guard against re-entry on a second signal during shutdown
    $_SHUTTING_DOWN && return
    _SHUTTING_DOWN=true

    echo ""
    echo "[entrypoint] Shutting down..."

    for pidfile in "$PIDDIR"/*.pid; do
        [ -f "$pidfile" ] || continue
        pid=$(cat "$pidfile")
        if kill -0 "$pid" 2>/dev/null; then
            echo "[entrypoint] Sending SIGTERM to PID $pid"
            kill -TERM "$pid" 2>/dev/null || true
        fi
    done

    local timeout=10
    while [ "$timeout" -gt 0 ]; do
        local all_dead=true
        for pidfile in "$PIDDIR"/*.pid; do
            [ -f "$pidfile" ] || continue
            pid=$(cat "$pidfile")
            kill -0 "$pid" 2>/dev/null && all_dead=false && break
        done
        $all_dead && break
        sleep 1
        timeout=$((timeout - 1))
    done

    for pidfile in "$PIDDIR"/*.pid; do
        [ -f "$pidfile" ] || continue
        pid=$(cat "$pidfile")
        if kill -0 "$pid" 2>/dev/null; then
            echo "[entrypoint] Force killing PID $pid"
            kill -KILL "$pid" 2>/dev/null || true
        fi
    done

    echo "[entrypoint] Shutdown complete."
    exit 0
}

trap cleanup SIGTERM SIGINT SIGQUIT

# Poll a TCP port until it accepts connections, max 15 seconds
wait_for_port() {
    local port=$1 name=$2
    for _ in $(seq 1 30); do
        echo > /dev/tcp/127.0.0.1/"$port" 2>/dev/null && return 0
        sleep 0.5
    done
    echo "[entrypoint] Timed out waiting for $name on port $port" >&2
    exit 1
}

# Start config-server and wait until it is ready
echo "[entrypoint] Starting perlica-config-server (0.0.0.0:21041)..."
perlica-config-server &
echo $! > "$PIDDIR/config-server.pid"
wait_for_port 21041 config-server

# Start game-server and wait until it is ready
echo "[entrypoint] Starting perlica-game-server (game: ${GAME_HOST}:${GAME_PORT}, GM bridge: ${GM_BRIDGE_HOST}:${GM_BRIDGE_PORT})..."
perlica-game-server "$GAME_CONFIG" &
echo $! > "$PIDDIR/game-server.pid"
wait_for_port "$GAME_PORT" game-server

# Start muip admin panel
echo "[entrypoint] Starting perlica-muip-server (admin panel: 0.0.0.0:${MUIP_PORT})..."
perlica-muip-server "$MUIP_CONFIG" &
echo $! > "$PIDDIR/muip-server.pid"

echo ""
echo "============================================"
echo "  Config Server:  0.0.0.0:21041"
echo "  Game Server:    ${GAME_HOST}:${GAME_PORT}"
echo "  GM Bridge:      ${GM_BRIDGE_HOST}:${GM_BRIDGE_PORT} (internal)"
echo "  Admin Panel:    0.0.0.0:${MUIP_PORT}"
echo "  Saves:          /app/saves/"
echo "============================================"
echo ""

# Monitor children and trigger cleanup if any exit unexpectedly
while true; do
    for pidfile in "$PIDDIR"/*.pid; do
        [ -f "$pidfile" ] || continue
        pid=$(cat "$pidfile")
        if ! kill -0 "$pid" 2>/dev/null; then
            procname=$(basename "$pidfile" .pid)
            echo "[entrypoint] $procname (PID $pid) exited unexpectedly!"
            cleanup
        fi
    done
    sleep 2
done
