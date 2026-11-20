#!/usr/bin/env bash
# Fails if ed25519-dalek or blake2 is reachable from a shipping build.
#
# usher-testkit carries both crates for its test signer. Every other workspace
# member, found from `cargo metadata` so a new crate is covered without an
# edit, must show neither in `cargo tree -e normal,build`. An empty member list
# or a tree that does not name its member also fails: a check that saw nothing
# must not pass.
#
# usage: scripts/guard-signer-crates.sh [workspace-dir]
# GUARD_CARGO names the cargo to run (default: cargo); the fixture test uses it
# to feed the guard a tree that does not name its member.
set -euo pipefail

cargo_bin="${GUARD_CARGO:-cargo}"

cd "${1:-.}"

names="$("$cargo_bin" metadata --no-deps --format-version 1 | jq -r '.packages[].name')"
members="$(printf '%s\n' "$names" | grep -vx 'usher-testkit' || true)"
if [[ -z "$members" ]]; then
  echo "guard: no workspace member other than usher-testkit was found" >&2
  exit 1
fi

status=0
while IFS= read -r member; do
  tree="$("$cargo_bin" tree --locked -e normal,build --target all --all-features -p "$member" --prefix none)"
  if ! printf '%s\n' "$tree" | grep -q "^${member} "; then
    echo "guard: cargo tree did not list ${member}" >&2
    status=1
    continue
  fi
  if hits="$(printf '%s\n' "$tree" | grep -E '^(ed25519-dalek|blake2) v')"; then
    echo "guard: ${member} reaches a test-only signing crate:" >&2
    printf '%s\n' "$hits" >&2
    status=1
  fi
done <<< "$members"
exit "$status"
