#!/bin/sh
set -eu
root="$(cd "$(dirname "$0")/.." && pwd)"
dst="$root/android/app/src/main/assets/www"
mkdir -p "$dst/funo/components"
cp "$root/web/index.html" "$root/web/app.js" "$root/web/funo.js" "$root/web/funo-bundle.js" "$root/web/style.css" "$dst/"
cp "$root/funo/components/"*.fun "$dst/funo/components/"
echo "packed assets into $dst"
