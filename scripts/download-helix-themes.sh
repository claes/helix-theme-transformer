#!/usr/bin/env bash
set -euo pipefail

repo_url="https://github.com/helix-editor/helix.git"
out_dir="helix-themes"
tmp_dir="$(mktemp -d)"

cleanup() {
  rm -rf "$tmp_dir"
}

trap cleanup EXIT

git clone \
  --depth 1 \
  --filter=blob:none \
  --sparse \
  "$repo_url" \
  "$tmp_dir/helix"

git -C "$tmp_dir/helix" sparse-checkout set runtime/themes

rm -rf "$out_dir"
mkdir -p "$out_dir"
cp -R "$tmp_dir/helix/runtime/themes/." "$out_dir/"

echo "Downloaded Helix themes to $out_dir"
