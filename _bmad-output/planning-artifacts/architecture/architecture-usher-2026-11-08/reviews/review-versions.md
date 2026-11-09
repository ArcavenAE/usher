# Review: Stack versions and technology claims

Spine: `ARCHITECTURE-SPINE.md` (Stack table, AD-2, AD-5, AD-9, AD-12, AD-13).
Checked 2026-11-08 against the crates.io API, static.rust-lang.org, doc.rust-lang.org,
docs.rs and project sources. Every crates.io call used `curl -A 'usher-review ...'`.

Verdict: every Stack version is the current stable release and every crate is maintained;
two claims need a qualifier (rmcp stdio needs a non-default feature, notify on macOS uses
FSEvents only with its default feature), and the plan to write usher's own JSON and YAML span
editor should be revisited because maintained format-preserving crates exist.

## 1. Stack versions are current and maintained

| Name | Spine | crates.io max_stable (2026-11-08) | Last release | Verdict |
| --- | --- | --- | --- | --- |
| Rust stable | 1.99.1 | 1.99.1 (58cf8f9c3 2026-10-13), channel date 2026-10-15 | 2026-10-13 | confirmed |
| tokio | 1.54.1 | 1.54.1 | 2026-10-21 | confirmed |
| rusqlite | 0.41.1 | 0.41.1 (rust-version 1.89.0) | 2026-10-28 | confirmed |
| toml_edit | 0.25.19 | 0.25.19+spec-1.1.0 | 2026-11-06 | confirmed |
| serde_json | 1.0.154 | 1.0.154 | 2026-10-11 | confirmed |
| notify | 8.2.2 | 8.2.2 (9.0.0-rc.7 also published 2026-10-27) | 2026-11-08 | confirmed, see note |
| rmcp | 3.9.0 | 3.9.0, repo modelcontextprotocol/rust-sdk | 2026-11-05 | confirmed |
| axum | 0.8.9 | 0.8.9 | 2026-04-14 | confirmed |
| minisign-verify | 0.3.0 | 0.3.0 | 2026-09-25 | confirmed |

Evidence: `https://crates.io/api/v1/crates/<name>` for each crate (fields `max_stable_version`,
`updated_at`, `repository`); `https://static.rust-lang.org/dist/channel-rust-stable.toml`
(`[pkg.rust] version = "1.99.1 (58cf8f9c3 2026-10-13)"`). All MSRVs (highest is rusqlite at
1.89.0; notify and rmcp at 1.88) are below 1.99.1.

Notes:
- notify 8.2.2 was published on the day of this review, and the 9.0 line is at rc.7
  (`https://crates.io/api/v1/crates/notify/versions`). A 9.0 stable is likely soon and may
  change the watcher API.
- axum's last release is about seven months old; its repo (tokio-rs/axum) is active, and 0.8
  is the current line. Not a concern.
- toml_edit's version string carries build metadata (`+spec-1.1.0`). Cargo ignores it in
  requirements, so `toml_edit = "0.25.19"` resolves.

Fix: none required for the table. Optionally record that notify 9.0 is in release candidate,
so a builder pins `notify = "8.2"` and treats the 9.0 move as a planned upgrade.

## 2. rmcp supports a stdio MCP server

Claim: rmcp (official MCP SDK) serves MCP mode over stdio.

Evidence: the rmcp 3.9.0 README shows `use rmcp::{..., transport::stdio}` and
`Calculator.serve(stdio()).await?`, with the stdio server example
`examples/servers/src/calculator_stdio.rs`
(`https://crates.io/api/v1/crates/rmcp/3.9.0/readme`). The `stdio` export is gated:
`#[cfg(feature = "transport-io")] pub use io::{StdioReader, StdioWriter, stdio, stdio_pipes};`
(`https://raw.githubusercontent.com/modelcontextprotocol/rust-sdk/main/crates/rmcp/src/transport.rs`).
The 3.9.0 feature map (`https://crates.io/api/v1/crates/rmcp/3.9.0`) has
`default = [base64, macros, server]` and `transport-io = [transport-async-rw, tokio/io-std, tokio/net]`,
so `transport-io` is not on by default. The README's install line is only
`cargo add rmcp --features server`.

Verdict: confirmed, with a qualifier.

Fix: record the features in the Stack or AD that binds MCP mode, for example
`rmcp 3.9.0 (features: server, transport-io)`.

## 3. notify uses FSEvents on macOS and ReadDirectoryChangesW on Windows (AD-9)

Evidence: notify 8.2.2 README (`https://crates.io/api/v1/crates/notify/8.2.2/readme`) lists
"macOS: FSEvents or kqueue, see features" and "Windows: ReadDirectoryChangesW". The 8.2.2 feature
map has `default = [macos_fsevent]` and a separate `macos_kqueue`
(`https://crates.io/api/v1/crates/notify/8.2.2`).

Verdict: confirmed, on default features.

Fix: add "default features (macos_fsevent)" next to notify in AD-9 or the Stack, so nobody
disables defaults and lands on kqueue, which holds one file descriptor per watched file and
would strain the 256-directory budget.

## 4. toml_edit preserves comments and formatting on edit (AD-5)

Evidence: toml_edit 0.25.19 README
(`https://crates.io/api/v1/crates/toml_edit/0.25.19+spec-1.1.0/readme`): "documents, while
preserving comments, spaces and" relative order. Its Limitations section names two things it
does not preserve: the order of dotted keys, and the lack of a trailing newline.

Verdict: confirmed, with two documented exceptions.

Fix: AD-5 says "leaving every other byte unchanged". Either add a byte-equality check after a
toml_edit write (compare everything outside the target span with the original and refuse the
write if anything else moved), or note the two toml_edit exceptions so FR-49 tests cover them.

## 5. minisign-verify verifies minisign ed25519 signatures offline (AD-12)

Evidence: README (`https://crates.io/api/v1/crates/minisign-verify/0.3.0/readme`): "A small,
zero-dependencies Rust crate to verify Minisign signatures", supporting standard and
pre-hashed modes and `PublicKey::from_base64(...)` for a key held in code. Source
(`https://raw.githubusercontent.com/jedisct1/rust-minisign-verify/master/src/lib.rs`) imports
`crate::crypto::ed25519` and only `std::{fmt, fs, io}` and `std::path`. The dependency
endpoint (`https://crates.io/api/v1/crates/minisign-verify/0.3.0/dependencies`) returns an
empty list, so there is no network client in the crate. The author (jedisct1) is the author
of minisign.

Verdict: confirmed. A key compiled in with `PublicKey::from_base64` fits AD-12.

Fix: none. Optional: state whether usher accepts both standard and pre-hashed signatures or
only one, since `verify` takes a flag for legacy (non-prehashed) signatures.

## 6. Does std::fs::File::lock replace a locking crate, and does it map to flock and LockFileEx?

Evidence: `https://doc.rust-lang.org/stable/std/fs/struct.File.html`, method `lock`, marked
"Stable since Rust version 1.89.0": "This function currently corresponds to the flock function
on Unix with the LOCK_EX flag, and the LockFileEx function on Windows with the
LOCKFILE_EXCLUSIVE_LOCK flag. Note that, this may change in the future." `try_lock` maps to
`LOCK_EX | LOCK_NB` and `LOCKFILE_FAIL_IMMEDIATELY`. The lock is released when the handle and
its duplicates close. The docs also say the lock "may be advisory or mandatory", and that on
Windows locking fails if the file is opened only for append.

Verdict: confirmed. With Rust 1.99.1 no separate crate (fs2, fs4, fd-lock) is needed for the
journal lock.

Fix: name `std::fs::File::lock` and `try_lock` as the lock mechanism in the spine (the memlog
says "OS file lock (flock / LockFileEx)" but no AD names the API), open the lock file for
write and not append-only, and note that std reserves the right to change the mapping. Since
flock locks are per open file description, the lock must be held on one handle for the whole
intent-to-outcome span, which AD-3 already implies.

## 7. Does a byte-preserving JSON or YAML editing crate exist? (AD-5)

Claim: AD-5 uses "usher's own span editor for JSON and YAML".

Evidence:
- JSON: `jsonc-parser` 0.34.0 (dprint, released 2026-09-27, about 3.4M recent downloads,
  `https://crates.io/api/v1/crates/jsonc-parser`) has a `cst` feature. Its docs
  (`https://docs.rs/jsonc-parser/0.34.0/jsonc_parser/cst/index.html`): "Unlike the AST, this
  keeps every comment and every piece of whitespace, so a document can be edited and written
  back out with everything the author wrote still in place", with `CstRootNode::parse`,
  `get(...).set_value(...)` and `append(...)`. It also parses JSONC, which Claude Code and other
  harness files may contain.
- YAML: `yaml-edit` 0.3.6 (jelmer, released 2026-10-31,
  `https://github.com/jelmer/yaml-edit`), "parsing and editing YAML files while preserving
  formatting, comments, and whitespace", built on rowan lossless trees. `yamlpatch` 1.32.0
  (zizmor project, released 2026-11-02,
  `https://github.com/zizmorcore/zizmor/tree/main/crates/yamlpatch`), "Comment and
  format-preserving YAML patch operations" for targeted changes, on top of `yamlpath`.

Verdict: wrong as a plan of record, in that crates exist that cover most of it. Whether each one
leaves every byte outside the target unchanged is unconfirmed here; none of the three states a
byte-for-byte guarantee, and yaml-edit is pre-1.0.

Fix: change AD-5 to "`jsonc-parser` (cst) for JSON; `yamlpatch` or `yaml-edit` for YAML; usher's
own span editor only where a fixture shows one of them moving bytes outside the target". Keep
the byte-equality check from item 4 as the guard for all three formats. Add the chosen crates
to the Stack table with versions.

## 8. rusqlite `bundled` still ships SQLite

Evidence: rusqlite 0.41.1 features (`https://crates.io/api/v1/crates/rusqlite/0.41.1`):
`bundled = [libsqlite3-sys?/bundled, modern_sqlite]`. The libsqlite3-sys tree carries the
amalgamation: `upgrade.sh` sets `SQLITE=sqlite-amalgamation-3540100` and `sqlite3/sqlite3.h`
defines `SQLITE_VERSION "3.54.1"`
(`https://raw.githubusercontent.com/rusqlite/rusqlite/master/libsqlite3-sys/sqlite3/sqlite3.h`,
read from master, not the 0.39.1 tag). Current libsqlite3-sys is 0.39.1, released 2026-10-25.

Verdict: confirmed. The SQLite version inside the 0.41.1 release itself is unconfirmed (master
was read, not the tag).

Fix: none required. Note that `bundled` is not in rusqlite's default features
(`default = [cache, ffi-sqlite-wasm-rs]`), so the Stack entry "rusqlite (bundled SQLite)"
should read as `features = ["bundled"]`; on Windows, `bundled-windows` is an alternative only if
a system SQLite is wanted, which the spine does not want.

## Not checked

- AD-2 and AD-13 make no crate claim beyond the Stack (axum for collector HTTPS). The TLS stack
  for `usher collector` HTTPS (rustls via axum-server or similar) is not named in the Stack;
  axum alone does not terminate TLS. Fix: name the TLS crate and version, or state that TLS
  ends at a proxy.
