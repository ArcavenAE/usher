---
title: 'Installed tools, their source, version and whether they are behind (Story 2.1)'
type: 'feature'
created: '2026-11-20'
status: 'in-progress'
route: 'dispatch'
---

## Intent

**Problem:** a person cannot see which fleet tools and harnesses are installed, where each came from and which version is on disk.

**Approach:** `usher tools` reads brew and mise state from the files those installers already wrote and prints one row per tool. The first change runs no installer. The behind-check comes later and goes through the network port.

## Invariant test scope

A read-only verb's invariant test watches the real installer state on every host and watches harness homes in scratch copies. A harness home is watched in its real location only by the first change whose verb reads that home, and that test must say how it tells usher's writes apart from those of a harness session running on the same host.

In this change `usher tools` reads installer files only and opens no harness home. Its invariant test therefore watches the real installer state (the brew repository head and fetch head, taps, Cellar, Caskroom, the linked and opt links, the brew api cache, the mise installs, shims, cache and state directories) and watches harness homes only as scratch copies built by the test. The real harness homes are left out because a harness session on the same host writes there while the test runs, and a change would be blamed on the verb by mistake. A brew or mise run by the person while the test runs reads as a violation, which is a true report and not a flake to retry.
