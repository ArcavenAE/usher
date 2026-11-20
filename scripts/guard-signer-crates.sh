#!/usr/bin/env bash
# Fails if ed25519-dalek or blake2 is reachable from a shipping build.
#
# usher-testkit carries both crates for its test signer, and usher-probes
# reaches them through it. Every other workspace member, found from
# `cargo metadata` so a new crate is covered without an edit, must show neither in `cargo tree -e normal,build`. An empty member list
# or a tree that does not name its member also fails: a check that saw nothing
# must not pass.
#
# usage: scripts/guard-signer-crates.sh [workspace-dir]
# GUARD_CARGO names the cargo to run (default: cargo); the fixture test uses it
# to feed the guard a tree that does not name its member.
set -euo pipefail

cargo_bin="${GUARD_CARGO:-cargo}"

cd "${1:-.}"

# The two test-only members. usher-testkit carries the signing crates for its
# test signer, and usher-probes takes the testkit as a normal dependency, so it
# reaches them through it. Nothing else is exempt, and the list is not derived
# from `publish`, which the workspace sets for every crate. A shipping member
# that depended on either would reach the signing crates and fail here.
exempt=(usher-testkit usher-probes)

names="$("$cargo_bin" metadata --no-deps --format-version 1 | jq -r '.packages[].name')"
members="$(printf '%s\n' "$names" | grep -vxF -f <(printf '%s\n' "${exempt[@]}") || true)"
if [[ -z "$members" ]]; then
  echo "guard: no workspace member other than ${exempt[*]} was found" >&2
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
