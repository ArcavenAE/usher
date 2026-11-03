# local probe: codex app-server protocol schema (read-only), 2026-11-03, kinu, codex-cli 0.160.1

Command: `codex app-server generate-json-schema --out <scratch dir>` (writes schema files to a scratch directory only; reads no config values).
File: codex_app_server_protocol.v2.schemas.json

- ConfigReadParams: `cwd` ("Optional working directory to resolve project config layers. If specified, return the effective config as seen from that directory (i.e., including any project layers between `cwd` and the project/repo root)."), `includeLayers` (boolean).
- ConfigReadResponse: properties `config`, `layers`, `origins`; required `config`, `origins`. `origins` is an object mapping each key to ConfigLayerMetadata {name, version}.
- ConfigLayer: `config`, `disabledReason` (string or null), `name`, `version`.
- ConfigLayerSource variants: packagedDefaults, mdm, system, enterpriseManaged, user, project, sessionFlags, legacyManagedConfigTomlFromFile, legacyManagedConfigTomlFromMdm.
- ConfigRequirementsReadResponse: `requirements`.
