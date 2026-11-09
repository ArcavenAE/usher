# Capability scope

Release per capability from PRD §7, and the spine decisions that govern each. Release is the shipping order only; it does not change what a capability is. "v2" items are sequenced, not dropped.

| CAP | FRs | Release | Governed by |
| --- | --- | --- | --- |
| CAP-1 | FR-43, FR-1 to FR-8 | MVP | AD-6, AD-8, AD-10, AD-17, AD-18 |
| CAP-2 | FR-9 to FR-12, FR-44, FR-45 | MVP | AD-2, AD-3, AD-4, AD-15, AD-19 (self-update through the owning installer) |
| CAP-3 | FR-46 to FR-48 | MVP | AD-1, AD-15 |
| CAP-4 | FR-65 | v2 | none yet |
| CAP-5 | FR-66 | v2 | AD-22 |
| CAP-6 | FR-67 | MVP | AD-3, AD-15, AD-19 |
| CAP-7 | FR-13, FR-14 | MVP | AD-9, AD-18 |
| CAP-8 | FR-15 to FR-18 | MVP | AD-6, AD-8, AD-12, AD-17 |
| CAP-9 | FR-19 to FR-23 | MVP | AD-11 |
| CAP-10 | FR-24, FR-25, FR-49 to FR-52 | MVP, on the host usher runs on; remote apply v2 | AD-2, AD-3, AD-5, AD-11, AD-16 |
| CAP-11 | FR-26, FR-27 / FR-53 | MVP / v2 | AD-8 |
| CAP-12 | FR-28, FR-29 / FR-54 | MVP / v2 | AD-13, AD-14 |
| CAP-13 | FR-30 to FR-36 | MVP | AD-6, AD-12, AD-20 |
| CAP-14 | FR-37 to FR-42 | MVP, receive and query only, one affiliation per collector | AD-13, AD-14 |
| CAP-15 | FR-55 / FR-56 | MVP, profiles and own records / v2, templates | AD-11 |
| CAP-16 | FR-57, FR-58 | v2+ | none yet |
| CAP-17 | FR-59 to FR-61 | v2 | AD-13 is the seam |
| CAP-18 | FR-62 | MVP: CLI, background service, thin MCP surface / v2: full in-LLM surface and app | AD-1, AD-3, AD-4, AD-16, AD-21 (app) |
| CAP-19 | FR-63, FR-64 | MVP, every change the MVP makes | AD-2, AD-3, AD-5, AD-7, AD-8 |

MVP harnesses: Claude Code, the Claude desktop app with Claude's general settings, codex and goose. Others enter through harness knowledge. Hazard checks ship only as their tests pass.

Later, beyond v2: several affiliations per person, the marvel policy fragment, AI at the endpoint and for analysis, chat products.
