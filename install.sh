#!/bin/sh
set -eu

notify_repository="${DISCORD_NOTIFICATION_REPOSITORY:-memset0/discord-notification}"
notify_install_dir="${DISCORD_NOTIFICATION_INSTALL_DIR:-}"
notify_version="${DISCORD_NOTIFICATION_VERSION:-}"

notify_detect_target() {
    notify_os=$1
    notify_arch=$2

    case "$notify_os" in
        Linux)
            case "$notify_arch" in
                x86_64 | amd64) printf '%s\n' "x86_64-unknown-linux-musl" ;;
                aarch64 | arm64) printf '%s\n' "aarch64-unknown-linux-musl" ;;
                *)
                    printf 'unsupported Linux architecture: %s\n' "$notify_arch" >&2
                    return 1
                    ;;
            esac
            ;;
        Darwin)
            case "$notify_arch" in
                x86_64 | amd64) printf '%s\n' "x86_64-apple-darwin" ;;
                aarch64 | arm64) printf '%s\n' "aarch64-apple-darwin" ;;
                *)
                    printf 'unsupported macOS architecture: %s\n' "$notify_arch" >&2
                    return 1
                    ;;
            esac
            ;;
        *)
            printf 'unsupported operating system: %s\n' "$notify_os" >&2
            return 1
            ;;
    esac
}

if [ "${1:-}" = "--print-target" ]; then
    notify_detect_target "${2:-$(uname -s)}" "${3:-$(uname -m)}"
    exit 0
fi

if ! command -v curl >/dev/null 2>&1; then
    printf '%s\n' "error: curl is required to download a release" >&2
    exit 1
fi
if ! command -v tar >/dev/null 2>&1; then
    printf '%s\n' "error: tar is required to unpack a release" >&2
    exit 1
fi

notify_target="${DISCORD_NOTIFICATION_TARGET:-$(notify_detect_target "$(uname -s)" "$(uname -m)")}"

if [ -z "$notify_version" ]; then
    notify_release_json=$(curl --proto '=https' --tlsv1.2 -fsSL \
        "https://api.github.com/repos/${notify_repository}/releases/latest")
    notify_version=$(printf '%s\n' "$notify_release_json" |
        sed -n 's/.*"tag_name":[[:space:]]*"\([^"]*\)".*/\1/p' |
        sed -n '1p')
    if [ -z "$notify_version" ]; then
        printf '%s\n' "error: could not determine the latest release version" >&2
        exit 1
    fi
fi

case "$notify_version" in
    v*) notify_tag=$notify_version ;;
    *) notify_tag="v${notify_version}" ;;
esac

if [ -z "$notify_install_dir" ]; then
    if [ -z "${HOME:-}" ]; then
        printf '%s\n' "error: HOME is not set; set DISCORD_NOTIFICATION_INSTALL_DIR" >&2
        exit 1
    fi
    notify_install_dir="${HOME}/.local/bin"
fi

notify_archive="notify-me-on-discord-${notify_tag}-${notify_target}.tar.gz"
notify_download_base="https://github.com/${notify_repository}/releases/download/${notify_tag}"
notify_temp_dir=$(mktemp -d)
notify_cleanup() {
    rm -rf -- "$notify_temp_dir"
}
trap notify_cleanup EXIT HUP INT TERM
umask 077

printf 'Downloading %s\n' "$notify_archive"
curl --proto '=https' --tlsv1.2 -fsSL \
    "${notify_download_base}/${notify_archive}" \
    -o "${notify_temp_dir}/${notify_archive}"
curl --proto '=https' --tlsv1.2 -fsSL \
    "${notify_download_base}/${notify_archive}.sha256" \
    -o "${notify_temp_dir}/${notify_archive}.sha256"

notify_expected=$(sed -n '1s/[[:space:]].*//p' "${notify_temp_dir}/${notify_archive}.sha256")
case "$notify_expected" in
    *[!0-9a-fA-F]* | '') printf '%s\n' "error: release checksum is malformed" >&2; exit 1 ;;
esac

if command -v sha256sum >/dev/null 2>&1; then
    notify_actual=$(sha256sum "${notify_temp_dir}/${notify_archive}" |
        sed -n '1s/[[:space:]].*//p')
elif command -v shasum >/dev/null 2>&1; then
    notify_actual=$(shasum -a 256 "${notify_temp_dir}/${notify_archive}" |
        sed -n '1s/[[:space:]].*//p')
else
    printf '%s\n' "error: sha256sum or shasum is required to verify the release" >&2
    exit 1
fi

if [ "$(printf '%s' "$notify_expected" | tr 'A-F' 'a-f')" != \
    "$(printf '%s' "$notify_actual" | tr 'A-F' 'a-f')" ]; then
    printf '%s\n' "error: release checksum mismatch; nothing was installed" >&2
    exit 1
fi

mkdir -p "${notify_temp_dir}/unpacked"
tar -xzf "${notify_temp_dir}/${notify_archive}" -C "${notify_temp_dir}/unpacked"
for notify_binary in notify-me-on-discord pingme; do
    if [ ! -f "${notify_temp_dir}/unpacked/${notify_binary}" ]; then
        printf 'error: release archive does not contain %s\n' "$notify_binary" >&2
        exit 1
    fi
    chmod 755 "${notify_temp_dir}/unpacked/${notify_binary}"
done

mkdir -p "$notify_install_dir"
for notify_binary in notify-me-on-discord pingme; do
    notify_temporary_destination="${notify_install_dir}/.${notify_binary}.tmp.$$"
    cp "${notify_temp_dir}/unpacked/${notify_binary}" "$notify_temporary_destination"
    chmod 755 "$notify_temporary_destination"
    mv -f "$notify_temporary_destination" "${notify_install_dir}/${notify_binary}"
done

printf 'Installed notify-me-on-discord %s to %s\n' "$notify_tag" "$notify_install_dir"
case ":${PATH:-}:" in
    *":${notify_install_dir}:"*) ;;
    *)
        printf 'Add %s to PATH, then run: pingme --help\n' "$notify_install_dir"
        ;;
esac
printf '%s\n' "Initialize portable files with: notify-me-on-discord init --portable"
