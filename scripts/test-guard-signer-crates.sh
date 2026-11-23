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
  if [[ -n "$needle" ]] && ! printf '%s\n' "$out" | grep -qF -- "$needle"; then
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

# usher-probes is the second test-only member: it takes the testkit as a normal
# dependency, so it reaches the signing crates through it.
stub member usher-probes usher-testkit:member
workspace usher-testkit usher-probes clean
expect pass "usher-probes may reach a signing crate through the testkit"

stub member shipper usher-probes:member
workspace usher-testkit usher-probes clean shipper
expect fail "a normal dependency on usher-probes still fails" "shipper reaches"

stub member bad blake2:vendor
workspace usher-testkit clean bad
expect fail "a member with a signing crate as a normal dependency fails" "bad reaches"

stub member deep ed25519-dalek:vendor
stub member via deep:member
workspace usher-testkit clean deep via
expect fail "a signing crate reached through another crate fails" "via reaches"

workspace usher-testkit
expect fail "no member other than usher-testkit fails" "no workspace member"

# A cargo whose `tree` output does not name the member (and shows no signing
# crate) must still fail: a check that saw nothing must not pass.
cat > "$scratch/blind-cargo" <<'WRAPPER'
#!/usr/bin/env bash
if [[ "$1" == tree ]]; then
  echo "unrelated v0.0.0"
else
  exec cargo "$@"
fi
WRAPPER
chmod +x "$scratch/blind-cargo"
workspace usher-testkit clean
GUARD_CARGO="$scratch/blind-cargo" expect fail "a tree that does not list the member fails" "did not list clean"

# The test-seam feature must not be enabled by any shipping member's normal
# dependency tree. `seamed` declares the feature; `leaky` enables it as a normal
# dependency; `quiet` depends on it without the feature.
seamed() {
  mkdir -p "$scratch/ws/seamed/src"
  printf '[package]\nname = "seamed"\nversion = "0.0.0"\nedition = "2024"\n[features]\ntest-seam = []\n' > "$scratch/ws/seamed/Cargo.toml"
  : > "$scratch/ws/seamed/src/lib.rs"
}
depends_on_seamed() { # depends_on_seamed <member> <with|without>
  mkdir -p "$scratch/ws/$1/src"
  {
    printf '[package]\nname = "%s"\nversion = "0.0.0"\nedition = "2024"\n[dependencies]\n' "$1"
    if [[ "$2" == with ]]; then
      printf 'seamed = { path = "../seamed", features = ["test-seam"] }\n'
    else
      printf 'seamed = { path = "../seamed" }\n'
    fi
  } > "$scratch/ws/$1/Cargo.toml"
  : > "$scratch/ws/$1/src/lib.rs"
}

seamed
depends_on_seamed quiet without
workspace usher-testkit clean seamed quiet
expect pass "a member that depends on a crate with a test-seam feature, without enabling it, passes"

depends_on_seamed leaky with
workspace usher-testkit clean seamed quiet leaky
expect fail "a member whose normal dependency enables test-seam fails" "leaky enables test-seam"

# An optional feature of a member can enable test-seam too: it shows only when
# the guard asks for all features.
mkdir -p "$scratch/ws/optional/src"
printf '[package]\nname = "optional"\nversion = "0.0.0"\nedition = "2024"\n[dependencies]\nseamed = { path = "../seamed" }\n[features]\nx = ["seamed/test-seam"]\n' > "$scratch/ws/optional/Cargo.toml"
: > "$scratch/ws/optional/src/lib.rs"
workspace usher-testkit clean seamed quiet optional
expect fail "a member whose optional feature enables test-seam fails" "optional enables test-seam"

# test-seam may be turned on only from [dev-dependencies]. A manifest scan, read
# from `cargo metadata`, names any other place, so a declaration the tree check
# cannot see (a build dependency, a feature of the member's own) is still refused.
seam_member() { # seam_member <member> <table> <with|without>
  mkdir -p "$scratch/ws/$1/src"
  {
    printf '[package]\nname = "%s"\nversion = "0.0.0"\nedition = "2024"\n[%s]\n' "$1" "$2"
    if [[ "$3" == with ]]; then
      printf 'seamed = { path = "../seamed", features = ["test-seam"] }\n'
    else
      printf 'seamed = { path = "../seamed" }\n'
    fi
  } > "$scratch/ws/$1/Cargo.toml"
  : > "$scratch/ws/$1/src/lib.rs"
}

seam_member devonly dev-dependencies with
workspace usher-testkit clean seamed devonly
expect pass "a member that enables test-seam from [dev-dependencies] passes"

seam_member normal dependencies with
workspace usher-testkit clean seamed normal
expect fail "test-seam from [dependencies] is named as outside [dev-dependencies]" "normal declares test-seam outside [dev-dependencies]"

seam_member builder build-dependencies with
workspace usher-testkit clean seamed builder
expect fail "test-seam from [build-dependencies] is named as outside [dev-dependencies]" "builder declares test-seam outside [dev-dependencies]"

mkdir -p "$scratch/ws/targeted/src"
printf '[package]\nname = "targeted"\nversion = "0.0.0"\nedition = "2024"\n[target."cfg(any())".dependencies]\nseamed = { path = "../seamed", features = ["test-seam"] }\n' > "$scratch/ws/targeted/Cargo.toml"
: > "$scratch/ws/targeted/src/lib.rs"
workspace usher-testkit clean seamed targeted
expect fail "test-seam from a target-specific dependency is named as outside [dev-dependencies]" "targeted declares test-seam outside [dev-dependencies]"

# The signing-crate exemption of usher-testkit does not reach the test-seam
# rule: the manifest scan reads every package, usher-testkit included.
mkdir -p "$scratch/ws/usher-testkit/src"
{
  printf '[package]\nname = "usher-testkit"\nversion = "0.0.0"\nedition = "2024"\n'
  printf '[dependencies]\nblake2 = { path = "../../vendor/blake2" }\n'
  printf 'seamed = { path = "../seamed", features = ["test-seam"] }\n'
} > "$scratch/ws/usher-testkit/Cargo.toml"
workspace usher-testkit clean seamed
expect fail "test-seam from usher-testkit's [dependencies] is named as outside [dev-dependencies]" "usher-testkit declares test-seam outside [dev-dependencies]"
