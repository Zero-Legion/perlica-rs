# Copyright (c) ÆthelingTeam and affiliates.
#
# This source code is licensed under the AGPL-3.0 license found in the
# LICENSE file in the root directory of this source tree.

# Stage 1: Build the application
FROM rust:nightly-bookworm AS builder

WORKDIR /build

# Install build dependencies (protoc is required by prost-build at compile time)
RUN apt-get update && \
    apt-get install -y --no-install-recommends protobuf-compiler && \
    rm -rf /var/lib/apt/lists/*

# Copy manifest files
COPY Cargo.toml Cargo.lock ./

# Create dummy workspace sources to cache dependencies without building real code
RUN mkdir -p lib/common/src   lib/config/src   lib/proto/src    \
             lib/logic/src    lib/db/src        lib/muip/src     \
             lib/codegen/src  servers/config-server/src          \
             servers/game-server/src servers/muip-server/src

RUN for f in lib/common/src/lib.rs lib/config/src/lib.rs lib/proto/src/lib.rs \
             lib/logic/src/lib.rs  lib/db/src/lib.rs     lib/muip/src/lib.rs  \
             lib/codegen/src/lib.rs; do echo "pub fn placeholder() {}" > "$f"; done

RUN for f in servers/config-server/src/main.rs servers/game-server/src/main.rs \
             servers/muip-server/src/main.rs; do echo "fn main() {}" > "$f"; done

# Copy member manifests so the workspace resolves correctly
COPY lib/common/Cargo.toml          lib/common/Cargo.toml
COPY lib/config/Cargo.toml          lib/config/Cargo.toml
COPY lib/proto/Cargo.toml           lib/proto/Cargo.toml
COPY lib/logic/Cargo.toml           lib/logic/Cargo.toml
COPY lib/db/Cargo.toml              lib/db/Cargo.toml
COPY lib/muip/Cargo.toml            lib/muip/Cargo.toml
COPY lib/codegen/Cargo.toml         lib/codegen/Cargo.toml
COPY servers/config-server/Cargo.toml servers/config-server/Cargo.toml
COPY servers/game-server/Cargo.toml   servers/game-server/Cargo.toml
COPY servers/muip-server/Cargo.toml   servers/muip-server/Cargo.toml

# Copy committed proto-generated code needed by lib/proto
COPY lib/proto/out lib/proto/out

# Build dependencies only, layer cache is invalidated only when Cargo.toml/Cargo.lock change
RUN cargo build --release || true

# Replace dummy sources with real source code
RUN rm -rf lib/common/src lib/config/src lib/proto/src lib/logic/src \
           lib/db/src lib/muip/src lib/codegen/src                    \
           servers/config-server/src servers/game-server/src servers/muip-server/src
COPY . .

RUN python3 - <<'EOF'
import json, os, shutil, subprocess

result = subprocess.run(
    ["cargo", "metadata", "--no-deps", "--format-version", "1"],
    capture_output=True, text=True, check=True
)
names = {p["name"] for p in json.loads(result.stdout)["packages"]}
variants = names | {n.replace("-", "_") for n in names}

fp_dir = "target/release/.fingerprint"
if os.path.isdir(fp_dir):
    for entry in os.listdir(fp_dir):
        prefix = entry.rsplit("-", 1)[0]
        if prefix in variants:
            shutil.rmtree(os.path.join(fp_dir, entry), ignore_errors=True)
            print(f"  purged fingerprint: {entry}")
EOF

# Build the real binaries (dependencies are already cached)
RUN cargo build --release --locked --workspace

# Stage 2: Runtime image
FROM debian:bookworm-slim AS runtime

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
      ca-certificates \
      netcat-openbsd \
      tini \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy compiled binaries from builder
COPY --from=builder /build/target/release/perlica-config-server /usr/local/bin/
COPY --from=builder /build/target/release/perlica-game-server   /usr/local/bin/
COPY --from=builder /build/target/release/perlica-muip-server   /usr/local/bin/

# Copy game assets and default config
COPY --from=builder /build/assets /app/assets
COPY --from=builder /build/servers/game-server/config.default.toml /app/config.default.toml

# Create non-root user and saves directory
RUN mkdir -p /app/saves && \
    useradd -m -d /app -s /sbin/nologin perlica && \
    chown -R perlica:perlica /app

# Copy entrypoint script
COPY docker-entrypoint.sh /docker-entrypoint.sh
RUN chmod +x /docker-entrypoint.sh

USER perlica

VOLUME ["/app/saves"]

EXPOSE 1337 8080 21041

HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
  CMD nc -z 127.0.0.1 1337 && nc -z 127.0.0.1 8080 && nc -z 127.0.0.1 21041 || exit 1

ENTRYPOINT ["tini", "--"]
CMD ["/docker-entrypoint.sh"]
