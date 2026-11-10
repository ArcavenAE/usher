#!/usr/bin/env bash
# Tests scripts/guard-signer-crates.sh against throwaway workspaces that use
# local stand-ins named like the signing crates, so no network is needed.
set -euo pipefail

guard="$(cd "$(dirname "$0")" && pwd)/guard-signer-crates.sh"
scratch="$(mktemp -d)"
trap 'rm -rf "$scratch"' EXIT

# Stand-ins for the signing crates live in vendor/, outside the workspace root,
# so cargo does not make them members.
stub() { # stub <member|vendor> <name> [dependency-name:<member|vendor>]
  local base="$scratch/ws" dir
  [[ "$1" == vendor ]] && base="$scratch/vendor"
  dir="$base/$2"
  mkdir -p "$dir/src"
  {
    printf '[package]\nname = "%s"\nversion = "0.0.0"\nedition = "2024"\n' "$2"
    printf '[dependencies]\n'
    if [[ -n "${3:-}" ]]; then
      local dep="${3%%:*}" kind="${3##*:}" path
      if [[ "$kind" == vendor ]]; then path="../../vendor/$dep"; else path="../$dep"; fi
      printf '%s = { path = "%s" }\n' "$dep" "$path"
    fi
  } > "$dir/Cargo.toml"
  : > "$dir/src/lib.rs"
}

workspace() { # workspace <members...>
  local members
  members="$(printf '"%s", ' "$@")"
  printf '[workspace]\nresolver = "3"\nmembers = [%s]\n' "${members%, }" > "$scratch/ws/Cargo.toml"
  rm -f "$scratch/ws/Cargo.lock"
  (cd "$scratch/ws" && cargo generate-lockfile --offline >/dev/null 2>&1)
}

expect() { # expect <pass|fail> <description> [text the output must contain]
  local want="$1" what="$2" needle="${3:-}" out rc=0
  out="$("$guard" "$scratch/ws" 2>&1)" || rc=$?
  if [[ "$want" == pass && "$rc" -ne 0 ]] || [[ "$want" == fail && "$rc" -eq 0 ]]; then
    echo "FAIL: $what (wanted $want, exit $rc)" >&2
    printf '%s\n' "$out" >&2
    exit 1
  fi
  if [[ -n "$needle" ]] && ! printf '%s\n' "$out" | grep -q "$needle"; then
    echo "FAIL: $what (output lacks: $needle)" >&2
    printf '%s\n' "$out" >&2
    exit 1
  fi
  echo "ok: $what"
}

mkdir -p "$scratch/ws" "$scratch/vendor"
stub vendor blake2
stub vendor ed25519-dalek
stub member usher-testkit blake2:vendor
stub member clean

workspace usher-testkit clean
expect pass "usher-testkit may carry a signing crate"

stub member bad blake2:vendor
workspace usher-testkit clean bad
expect fail "a member with a signing crate as a normal dependency fails" "bad reaches"

stub member deep ed25519-dalek:vendor
stub member via deep:member
workspace usher-testkit clean deep via
expect fail "a signing crate reached through another crate fails" "via reaches"

workspace usher-testkit
expect fail "no member other than usher-testkit fails" "no workspace member"
