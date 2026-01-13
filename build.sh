#!/usr/bin/env sh
# https://www.getzola.org/documentation/deployment/vercel/#using-a-custom-zola-binary

echo "${ZOLA_VERSION:-"latest"}" \
    | sed '/^latest$/!s/\(.*\)/tags\/v\1/' \
    | xargs -I% curl -fsSL "https://api.github.com/repos/getzola/zola/releases/%" \
    | grep -oP "\"browser_download_url\": ?\"\\K(.+linux-${ZOLA_LIBC:-"musl"}\\.tar\\.gz)" \
    | xargs curl -fsSL \
    | tar -xz
