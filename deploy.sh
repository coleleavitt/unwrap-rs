#!/usr/bin/env bash

set -Eeuo pipefail
IFS=$'\n\t'
umask 022

ROOT_DIR=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)

DEPLOY_HOST=${DEPLOY_HOST:-linode}
REMOTE_ROOT=${REMOTE_ROOT:-/opt/unwrap-rs}
SERVICE=${SERVICE:-unwrap-rs.service}
KEEP_RELEASES=${KEEP_RELEASES:-5}
PUBLIC_URL=${PUBLIC_URL:-https://unwrap.rs}
APP_PORT=${APP_PORT:-18080}
BUILD_TARGET=${BUILD_TARGET:-x86_64-unknown-linux-musl}
MIN_FREE_BYTES=${MIN_FREE_BYTES:-67108864}

SSH_OPTS=(-o BatchMode=yes -o ConnectTimeout=10)
RSYNC_RSH='ssh -o BatchMode=yes -o ConnectTimeout=10'
LOCAL_SERVER_PID=
TEMP_DIR=

log() {
    printf '[deploy] %s\n' "$*"
}

warn() {
    printf '[deploy] warning: %s\n' "$*" >&2
}

die() {
    printf '[deploy] error: %s\n' "$*" >&2
    exit 1
}

usage() {
    cat <<'EOF'
Usage:
  ./deploy.sh [deploy]
  ./deploy.sh check
  ./deploy.sh status
  ./deploy.sh rollback [release-id]
  ./deploy.sh --help

Commands:
  deploy      Build, smoke-test, and atomically deploy (default).
  check       Build and smoke-test without contacting production.
  status      Show the active service and release state.
  rollback    Activate `previous`, or a named release.

Environment overrides:
  DEPLOY_HOST   SSH host or alias       (default: linode)
  REMOTE_ROOT   Release root            (default: /opt/unwrap-rs)
  SERVICE       systemd unit            (default: unwrap-rs.service)
  KEEP_RELEASES Number of newest releases to retain (default: 5)
  PUBLIC_URL    Public origin           (default: https://unwrap.rs)
  APP_PORT      Loopback application port (default: 18080)
  BUILD_TARGET  Rust server target      (default: x86_64-unknown-linux-musl)
  MIN_FREE_BYTES Free-space reserve before upload (default: 64 MiB)
EOF
}

cleanup() {
    if [[ -n ${LOCAL_SERVER_PID:-} ]]; then
        kill "$LOCAL_SERVER_PID" 2>/dev/null || true
        wait "$LOCAL_SERVER_PID" 2>/dev/null || true
    fi
    if [[ -n ${TEMP_DIR:-} && -d $TEMP_DIR ]]; then
        rm -rf -- "$TEMP_DIR"
    fi
}
trap cleanup EXIT

require_commands() {
    local command
    for command in "$@"; do
        command -v "$command" >/dev/null 2>&1 || die "required command not found: $command"
    done
}

validate_config() {
    [[ $DEPLOY_HOST =~ ^[A-Za-z0-9._@:-]+$ ]] || die "unsafe DEPLOY_HOST: $DEPLOY_HOST"
    [[ $REMOTE_ROOT =~ ^/[A-Za-z0-9._/-]+$ && $REMOTE_ROOT != / ]] || die "unsafe REMOTE_ROOT: $REMOTE_ROOT"
    [[ $SERVICE =~ ^[A-Za-z0-9_.@-]+$ ]] || die "unsafe SERVICE: $SERVICE"
    [[ $KEEP_RELEASES =~ ^[0-9]+$ && $KEEP_RELEASES -ge 1 ]] || die "KEEP_RELEASES must be a positive integer"
    [[ $APP_PORT =~ ^[0-9]+$ && $APP_PORT -ge 1 && $APP_PORT -le 65535 ]] || die "APP_PORT must be between 1 and 65535"
    [[ $MIN_FREE_BYTES =~ ^[0-9]+$ ]] || die "MIN_FREE_BYTES must be a non-negative integer"
    [[ $BUILD_TARGET =~ ^[A-Za-z0-9._-]+$ ]] || die "unsafe BUILD_TARGET: $BUILD_TARGET"
    [[ $PUBLIC_URL =~ ^https?://[A-Za-z0-9.-]+(:[0-9]{1,5})?/?$ ]] || die "PUBLIC_URL must be a plain HTTP(S) origin"
}

public_host() {
    local value=${PUBLIC_URL#*://}
    value=${value%%/*}
    value=${value%%:*}
    printf '%s\n' "$value"
}

wait_for_local_health() {
    local base_url=$1
    local body
    for _ in $(seq 1 40); do
        body=$(curl --fail --silent --show-error --max-time 2 "$base_url/api/health" 2>/dev/null || true)
        if [[ $body == ok ]]; then
            return 0
        fi
        sleep 0.25
    done
    return 1
}

build_and_smoke_test() {
    require_commands cargo trunk curl file find python3 tar sha256sum git

    log "building static server ($BUILD_TARGET)"
    cargo build --locked --release --target "$BUILD_TARGET" -p server

    log "building frontend"
    (
        cd "$ROOT_DIR/frontend"
        env -u NO_COLOR trunk build --locked --release
    )

    local binary="$ROOT_DIR/target/$BUILD_TARGET/release/server"
    local dist="$ROOT_DIR/frontend/dist"
    [[ -x $binary ]] || die "server artifact is missing or not executable: $binary"
    file "$binary" | grep -Eq 'ELF 64-bit.*x86-64' || die "server artifact is not an x86-64 ELF binary"
    [[ -s $dist/index.html ]] || die "frontend index is missing"
    [[ -s $dist/robots.txt ]] || die "robots.txt is missing"
    [[ -s $dist/sitemap.xml ]] || die "sitemap.xml is missing"
    [[ -s $dist/social-card.png ]] || die "social-card.png is missing"

    local wasm_path
    wasm_path=$(find "$dist" -maxdepth 1 -type f -name '*.wasm' -print -quit)
    [[ -n $wasm_path ]] || die "frontend WASM artifact is missing"

    local smoke_port
    smoke_port=${SMOKE_PORT:-$(python3 - <<'PY'
import socket

with socket.socket() as sock:
    sock.bind(("127.0.0.1", 0))
    print(sock.getsockname()[1])
PY
)}
    local smoke_url="http://127.0.0.1:$smoke_port"
    local smoke_log="$TEMP_DIR/local-smoke.log"

    log "smoke-testing on $smoke_url"
    ADDR="127.0.0.1:$smoke_port" DIST_DIR="$dist" "$binary" >"$smoke_log" 2>&1 &
    LOCAL_SERVER_PID=$!

    if ! wait_for_local_health "$smoke_url"; then
        sed -n '1,160p' "$smoke_log" >&2 || true
        die "local health check failed"
    fi

    local response headers
    response=$(curl --fail --silent --show-error "$smoke_url/")
    [[ $response == *'<title>'* ]] || die "local HTML check failed"
    response=$(curl --fail --silent --show-error "$smoke_url/robots.txt")
    [[ $response == User-agent:* ]] || die "local robots.txt check failed"
    response=$(curl --fail --silent --show-error "$smoke_url/sitemap.xml")
    [[ $response == *'<urlset'* ]] || die "local sitemap check failed"
    headers=$(curl --fail --silent --show-error --head "$smoke_url/$(basename "$wasm_path")")
    grep -qi '^content-type: application/wasm' <<<"$headers" || die "local WASM content type check failed"
    [[ $(curl --silent --output /dev/null --write-out '%{http_code}' "$smoke_url/definitely-not-a-real-page") == 404 ]] || die "unknown paths must return 404"

    kill "$LOCAL_SERVER_PID"
    wait "$LOCAL_SERVER_PID" 2>/dev/null || true
    LOCAL_SERVER_PID=
}

make_release_archive() {
    local release_id=$1
    local stage="$TEMP_DIR/stage"
    local archive="$TEMP_DIR/$release_id.tar.gz"
    local checksum="$archive.sha256"
    local revision

    revision=$(git -C "$ROOT_DIR" rev-parse HEAD 2>/dev/null || printf 'unknown')
    install -d -m 0755 "$stage/bin"
    install -m 0755 "$ROOT_DIR/target/$BUILD_TARGET/release/server" "$stage/bin/server"
    cp -a "$ROOT_DIR/frontend/dist" "$stage/dist"
    printf 'release=%s\nrevision=%s\nbuilt_at=%s\n' \
        "$release_id" "$revision" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" >"$stage/REVISION"

    tar -C "$stage" -czf "$archive" .
    (
        cd "$TEMP_DIR"
        sha256sum "$(basename "$archive")" >"$(basename "$checksum")"
    )

    printf '%s\n%s\n' "$archive" "$checksum"
}

remote_prepare_upload() {
    local release_id=$1
    local archive_bytes=$2

    ssh "${SSH_OPTS[@]}" "$DEPLOY_HOST" bash -s -- \
        "$REMOTE_ROOT" "$SERVICE" "$release_id" "$archive_bytes" "$MIN_FREE_BYTES" "$APP_PORT" <<'REMOTE'
set -Eeuo pipefail
IFS=$'\n\t'

root=$1
service=$2
release_id=$3
archive_bytes=$4
reserve_bytes=$5
app_port=$6

for command in tar sha256sum systemctl curl flock find df nginx grep; do
    command -v "$command" >/dev/null 2>&1 || {
        printf 'missing remote command: %s\n' "$command" >&2
        exit 1
    }
done

[[ $(uname -m) == x86_64 ]] || {
    printf 'remote architecture must be x86_64\n' >&2
    exit 1
}
unit=$(systemctl cat "$service")
grep -Fq "ExecStart=$root/current/bin/server" <<<"$unit" || {
    printf 'systemd ExecStart does not match %s/current/bin/server\n' "$root" >&2
    exit 1
}
grep -Fq "Environment=ADDR=127.0.0.1:$app_port" <<<"$unit" || {
    printf 'systemd ADDR does not match port %s\n' "$app_port" >&2
    exit 1
}
grep -Fq "Environment=DIST_DIR=$root/current/dist" <<<"$unit" || {
    printf 'systemd DIST_DIR does not match %s/current/dist\n' "$root" >&2
    exit 1
}
systemctl is-enabled --quiet "$service"
nginx -t >&2
install -d -m 0755 "$root" "$root/releases" "$root/uploads"
[[ ! -e "$root/releases/$release_id" ]] || {
    printf 'release already exists: %s\n' "$release_id" >&2
    exit 1
}

available=$(df -B1 --output=avail "$root" | tail -n 1 | tr -d ' ')
required=$((archive_bytes * 3 + reserve_bytes))
if (( available < required )); then
    printf 'insufficient disk space: available=%s required=%s\n' "$available" "$required" >&2
    exit 1
fi

usage=$(df -P "$root" | awk 'NR == 2 { print $5 }')
if [[ ${usage%%%} -ge 90 ]]; then
    printf 'warning: filesystem containing %s is %s full\n' "$root" "$usage" >&2
fi

upload="$root/uploads/$release_id"
rm -rf -- "$upload"
install -d -m 0700 "$upload"
printf '%s\n' "$upload"
REMOTE
}

remote_activate() {
    local release_id=$1
    local origin_host=$2

    ssh "${SSH_OPTS[@]}" "$DEPLOY_HOST" bash -s -- \
        "$REMOTE_ROOT" "$SERVICE" "$release_id" "$APP_PORT" "$KEEP_RELEASES" "$PUBLIC_URL" "$origin_host" <<'REMOTE'
set -Eeuo pipefail
IFS=$'\n\t'

root=$1
service=$2
release_id=$3
app_port=$4
keep_releases=$5
public_url=${6%/}
origin_host=$7

releases="$root/releases"
upload="$root/uploads/$release_id"
archive="$upload/$release_id.tar.gz"
checksum="$archive.sha256"
incoming="$releases/.$release_id.incoming"
final="$releases/$release_id"

cleanup_remote() {
    rm -rf -- "$incoming" "$upload"
}
trap cleanup_remote EXIT

exec 9>"$root/.deploy.lock"
flock -n 9 || {
    printf 'another deployment is already running\n' >&2
    exit 1
}

[[ -s $archive && -s $checksum ]] || {
    printf 'uploaded archive or checksum is missing\n' >&2
    exit 1
}
(
    cd "$upload"
    sha256sum -c "$(basename "$checksum")"
)

rm -rf -- "$incoming"
install -d -m 0755 "$incoming"
tar -xzf "$archive" -C "$incoming"
[[ -x $incoming/bin/server && -s $incoming/dist/index.html ]] || {
    printf 'release contents are incomplete\n' >&2
    exit 1
}
[[ -s $incoming/dist/robots.txt && -s $incoming/dist/sitemap.xml && -s $incoming/dist/social-card.png ]] || {
    printf 'SEO assets are incomplete\n' >&2
    exit 1
}

chown -R root:www-data "$incoming"
find "$incoming" -type d -exec chmod 0755 {} +
find "$incoming" -type f -exec chmod 0644 {} +
chmod 0755 "$incoming/bin/server"
mv -- "$incoming" "$final"

old_current=
if [[ -L $root/current || -e $root/current ]]; then
    old_current=$(readlink -f "$root/current")
fi
temp_link="$root/.current.$release_id.$$"
rm -f -- "$temp_link"
ln -s "$final" "$temp_link"
mv -Tf -- "$temp_link" "$root/current"

health_ok() {
    local body executable headers main_pid page robots sitemap wasm
    systemctl is-active --quiet "$service" || return 1
    main_pid=$(systemctl show --property MainPID --value "$service") || return 1
    [[ $main_pid =~ ^[1-9][0-9]*$ ]] || return 1
    executable=$(readlink -f "/proc/$main_pid/exe") || return 1
    [[ $executable == "$(readlink -f "$root/current/bin/server")" ]] || return 1
    body=$(curl --noproxy '*' --fail --silent --show-error --max-time 3 "http://127.0.0.1:$app_port/api/health" 2>/dev/null) || return 1
    [[ $body == ok ]] || return 1
    page=$(curl --noproxy '*' --fail --silent --show-error --max-time 3 "http://127.0.0.1:$app_port/") || return 1
    [[ $page == *'<title>'* ]] || return 1
    robots=$(curl --noproxy '*' --fail --silent --show-error --max-time 3 "http://127.0.0.1:$app_port/robots.txt") || return 1
    [[ $robots == User-agent:* ]] || return 1
    sitemap=$(curl --noproxy '*' --fail --silent --show-error --max-time 3 "http://127.0.0.1:$app_port/sitemap.xml") || return 1
    [[ $sitemap == *'<urlset'* ]] || return 1
    curl --noproxy '*' --fail --silent --show-error --max-time 3 "http://127.0.0.1:$app_port/social-card.png" >/dev/null || return 1
    wasm=$(find "$root/current/dist" -maxdepth 1 -type f -name '*.wasm' -printf '%f\n' -quit)
    [[ -n $wasm ]] || return 1
    headers=$(curl --noproxy '*' --fail --silent --show-error --head --max-time 3 "http://127.0.0.1:$app_port/$wasm") || return 1
    grep -qi '^content-type: application/wasm' <<<"$headers" || return 1
    if [[ $public_url == https://* ]]; then
        body=$(curl --noproxy '*' --resolve "$origin_host:443:127.0.0.1" --fail --silent --show-error --max-time 5 "$public_url/api/health" 2>/dev/null) || return 1
        [[ $body == ok ]] || return 1
    fi
}

healthy=0
if systemctl restart "$service"; then
    for _ in $(seq 1 30); do
        if health_ok; then
            healthy=1
            break
        fi
        sleep 1
    done
fi

if (( healthy == 0 )); then
    printf 'activation failed; restoring previous release\n' >&2
    if [[ -n $old_current && -d $old_current ]]; then
        rollback_link="$root/.rollback.$release_id.$$"
        ln -s "$old_current" "$rollback_link"
        mv -Tf -- "$rollback_link" "$root/current"
        systemctl restart "$service" || true
    else
        rm -f -- "$root/current"
        systemctl stop "$service" || true
    fi
    systemctl --no-pager --full status "$service" >&2 || true
    journalctl -u "$service" -n 80 --no-pager >&2 || true
    rm -rf -- "$final"
    exit 1
fi

    if [[ -n $old_current && $old_current != "$final" && -d $old_current ]]; then
        previous_link="$root/.previous.$release_id.$$"
        rm -f -- "$previous_link"
        ln -s "$old_current" "$previous_link"
    mv -Tf -- "$previous_link" "$root/previous"
fi

current_target=$(readlink -f "$root/current")
previous_target=
if [[ -L $root/previous || -e $root/previous ]]; then
    previous_target=$(readlink -f "$root/previous")
fi
kept=0
while IFS= read -r candidate; do
    [[ -n $candidate ]] || continue
    [[ $candidate == "$releases/"* ]] || continue
    if [[ $candidate == "$current_target" || $candidate == "$previous_target" ]]; then
        continue
    fi
    if (( kept < keep_releases )); then
        kept=$((kept + 1))
        continue
    fi
    rm -rf -- "$candidate"
done < <(find "$releases" -mindepth 1 -maxdepth 1 -type d ! -name '.*' -printf '%T@ %p\n' | sort -rn | cut -d' ' -f2-)

printf 'active_release=%s\n' "$release_id"
if [[ -n $old_current ]]; then
    printf 'previous_release=%s\n' "$(basename "$old_current")"
else
    printf 'previous_release=none\n'
fi
REMOTE
}

public_diagnostic() {
    local base=${PUBLIC_URL%/}
    local body=
    for _ in $(seq 1 10); do
        body=$(curl --fail --silent --show-error --max-time 10 "$base/api/health" 2>/dev/null || true)
        if [[ $body == ok ]] && \
            curl --fail --silent --show-error --max-time 10 "$base/robots.txt" | grep -q '^User-agent:' && \
            curl --fail --silent --show-error --max-time 10 "$base/sitemap.xml" | grep -q '<urlset'; then
            log "public checks passed: $base"
            return 0
        fi
        sleep 2
    done
    warn "the origin is healthy, but public checks did not converge for $base"
    return 0
}

deploy() {
    require_commands ssh rsync
    TEMP_DIR=$(mktemp -d -t unwrap-rs-deploy.XXXXXX)

    build_and_smoke_test

    local timestamp revision suffix release_id
    timestamp=$(date -u +%Y%m%dT%H%M%SZ)
    revision=$(git -C "$ROOT_DIR" rev-parse --short=12 HEAD 2>/dev/null || printf 'nogit')
    suffix=
    if [[ -n $(git -C "$ROOT_DIR" status --porcelain 2>/dev/null || true) ]]; then
        suffix=-dirty
    fi
    release_id="$timestamp-$revision$suffix"
    [[ $release_id =~ ^[A-Za-z0-9._-]+$ ]] || die "unsafe generated release ID"

    local archive checksum archive_bytes remote_upload
    local -a release_files
    mapfile -t release_files < <(make_release_archive "$release_id")
    archive=${release_files[0]}
    checksum=${release_files[1]}
    archive_bytes=$(stat -c '%s' "$archive")

    log "running remote preflight"
    remote_upload=$(remote_prepare_upload "$release_id" "$archive_bytes")
    log "uploading $release_id"
    rsync -az --protect-args -e "$RSYNC_RSH" -- \
        "$archive" "$checksum" "$DEPLOY_HOST:$remote_upload/"

    log "activating $release_id"
    remote_activate "$release_id" "$(public_host)"
    public_diagnostic
    log "deployment complete: $release_id"
}

check() {
    TEMP_DIR=$(mktemp -d -t unwrap-rs-check.XXXXXX)
    build_and_smoke_test
    log "build and local smoke checks passed"
}

status() {
    require_commands ssh curl
    ssh "${SSH_OPTS[@]}" "$DEPLOY_HOST" bash -s -- "$REMOTE_ROOT" "$SERVICE" "$APP_PORT" <<'REMOTE'
set -Eeuo pipefail
root=$1
service=$2
app_port=$3

printf 'service=%s\n' "$(systemctl is-active "$service")"
printf 'enabled=%s\n' "$(systemctl is-enabled "$service")"
if [[ -L $root/current || -e $root/current ]]; then
    printf 'current=%s\n' "$(readlink -f "$root/current")"
else
    printf 'current=none\n'
fi
if [[ -L $root/previous || -e $root/previous ]]; then
    printf 'previous=%s\n' "$(readlink -f "$root/previous")"
else
    printf 'previous=none\n'
fi
printf 'health=%s\n' "$(curl --noproxy '*' --fail --silent --show-error --max-time 3 "http://127.0.0.1:$app_port/api/health")"
df -h "$root"
find "$root/releases" -mindepth 1 -maxdepth 1 -type d ! -name '.*' -printf '%TY-%Tm-%Td %TH:%TM  %f\n' | sort -r
REMOTE
    public_diagnostic
}

rollback() {
    local requested=${1:-}
    local remote_requested=${requested:--}
    [[ -z $requested || $requested =~ ^[A-Za-z0-9._-]+$ ]] || die "unsafe release ID: $requested"
    require_commands ssh

    ssh "${SSH_OPTS[@]}" "$DEPLOY_HOST" bash -s -- \
        "$REMOTE_ROOT" "$SERVICE" "$APP_PORT" "$PUBLIC_URL" "$(public_host)" "$remote_requested" <<'REMOTE'
set -Eeuo pipefail
IFS=$'\n\t'

root=$1
service=$2
app_port=$3
public_url=${4%/}
origin_host=$5
requested=$6
if [[ $requested == - ]]; then
    requested=
fi
releases="$root/releases"

exec 9>"$root/.deploy.lock"
flock -n 9 || {
    printf 'another deployment is already running\n' >&2
    exit 1
}

[[ -L $root/current || -e $root/current ]] || {
    printf 'there is no active release to roll back\n' >&2
    exit 1
}
old_current=$(readlink -f "$root/current")
if [[ -n $requested ]]; then
    target=$(readlink -f "$releases/$requested" 2>/dev/null || true)
else
    target=
    if [[ -L $root/previous || -e $root/previous ]]; then
        target=$(readlink -f "$root/previous")
    fi
    if [[ -z $target ]]; then
        target=$(find "$releases" -mindepth 1 -maxdepth 1 -type d ! -name '.*' ! -path "$old_current" -printf '%T@ %p\n' | sort -rn | cut -d' ' -f2- | head -n 1)
    fi
fi

[[ -n $target && -d $target && $target == "$releases/"* ]] || {
    printf 'rollback target was not found\n' >&2
    exit 1
}
[[ $target != "$old_current" ]] || {
    printf 'rollback target is already active\n' >&2
    exit 1
}

temp_link="$root/.rollback.$$.tmp"
rm -f -- "$temp_link"
ln -s "$target" "$temp_link"
mv -Tf -- "$temp_link" "$root/current"
healthy=0
if systemctl restart "$service"; then
    for _ in $(seq 1 30); do
        main_pid=$(systemctl show --property MainPID --value "$service" 2>/dev/null || true)
        executable=
        if [[ $main_pid =~ ^[1-9][0-9]*$ ]]; then
            executable=$(readlink -f "/proc/$main_pid/exe" 2>/dev/null || true)
        fi
        if systemctl is-active --quiet "$service" && \
            [[ $executable == "$(readlink -f "$root/current/bin/server")" ]] && \
            [[ $(curl --noproxy '*' --fail --silent --show-error --max-time 3 "http://127.0.0.1:$app_port/api/health" 2>/dev/null || true) == ok ]] && \
            { [[ $public_url != https://* ]] || [[ $(curl --noproxy '*' --resolve "$origin_host:443:127.0.0.1" --fail --silent --show-error --max-time 5 "$public_url/api/health" 2>/dev/null || true) == ok ]]; }; then
            healthy=1
            break
        fi
        sleep 1
    done
fi

if (( healthy == 0 )); then
    restore_link="$root/.restore.$$.tmp"
    rm -f -- "$restore_link"
    ln -s "$old_current" "$restore_link"
    mv -Tf -- "$restore_link" "$root/current"
    systemctl restart "$service" || true
    systemctl --no-pager --full status "$service" >&2 || true
    exit 1
fi

previous_link="$root/.previous.$$.tmp"
rm -f -- "$previous_link"
ln -s "$old_current" "$previous_link"
mv -Tf -- "$previous_link" "$root/previous"
printf 'active_release=%s\n' "$(basename "$target")"
printf 'previous_release=%s\n' "$(basename "$old_current")"
REMOTE
    public_diagnostic
}

main() {
    validate_config
    cd "$ROOT_DIR"

    local command=${1:-deploy}
    case "$command" in
        deploy)
            [[ $# -eq 0 || $# -eq 1 ]] || die "deploy takes no arguments"
            deploy
            ;;
        check)
            [[ $# -eq 1 ]] || die "check takes no arguments"
            check
            ;;
        status)
            [[ $# -eq 1 ]] || die "status takes no arguments"
            status
            ;;
        rollback)
            [[ $# -le 2 ]] || die "rollback accepts at most one release ID"
            rollback "${2:-}"
            ;;
        -h|--help|help)
            usage
            ;;
        *)
            usage >&2
            die "unknown command: $command"
            ;;
    esac
}

main "$@"
