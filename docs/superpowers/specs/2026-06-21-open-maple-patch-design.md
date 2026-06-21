# Open Maple Patch Design

## Purpose

Open Maple Patch is an open-source, unofficial local patcher for MapleLegends text localization. It targets Windows 10/11 users who already have a legitimate MapleLegends client installed locally.

The first release provides a complete one-click assistant with a GUI, translation data updates, backup and restore, version validation, and developer tooling. It localizes static text plus quest title, quest description, and quest flow text. It does not localize NPC dialogue or story text by writing server-provided runtime content into client resources.

## Public Positioning

The project must be described as:

- Unofficial.
- Use at your own risk.
- Text-only localization.
- No gameplay advantage.
- Not affiliated with MapleLegends or Nexon.
- Not a bypass, automation tool, memory tool, packet tool, or patched client distributor.

The repository must not contain MapleLegends client files, original resource files, modified resource files, executable game files, or extracted paid localization data.

Suggested README wording:

```text
This project is an unofficial text localization patcher.
It does not distribute MapleLegends client files.
It does not modify gameplay values, map collision, character state, packets, memory, or automation behavior.
Use at your own risk.
```

## Target Scope

Initial target:

- Game: MapleLegends.
- Platform: Windows 10/11.
- App stack: Rust + Tauri.
- App shape: Complete GUI assistant.
- Update scope: Translation data auto-update only.
- Source layout: Single repository, structured so translation data can be split into a separate repository later.
- Localization modes: Simplified Chinese and Chinese-English bilingual.

Localized content:

- UI text.
- Item names and descriptions.
- Skill names and descriptions.
- Map names.
- Quest titles.
- Quest descriptions.
- Quest flow and quest info text.

Out of scope:

- NPC dialogue and story text fetched from the server at runtime.
- Memory injection.
- Packet reading, sniffing, editing, or replay.
- Automation, macro behavior, or input control.
- Map structure, collision, footholds, portals, spawn behavior, monster behavior, skill values, item stats, or any gameplay-affecting value.
- Distribution of original or modified MapleLegends client resources.
- Any use of paid localization package data as source material.

## Architecture

Use a monorepo with clear module boundaries:

```text
patch-core/
cli/
tauri-app/
translations/
fixtures/
docs/
```

`patch-core` is a Rust crate that owns all file and patching logic:

- Client directory detection.
- Required file detection.
- Resource hash calculation.
- Manifest matching.
- Translation data loading.
- Patch planning.
- Text length validation.
- Backup creation.
- Temporary output writing.
- Replacement of target files.
- Installation record writing.
- Restore logic.

`cli` is a developer and maintainer tool:

- Validate manifests and translation data.
- Run dry-run patch planning.
- Produce unknown-version reports.
- Enable developer mode for controlled experiments against unknown client versions.

`tauri-app` is the user-facing GUI:

- Select the MapleLegends folder.
- Show version and patch status.
- Install localization.
- Restore original files.
- Update translation data.
- Show logs and diagnostics.
- Manage language mode and settings.

`translations` contains public translation data and manifests:

- Simplified Chinese.
- Chinese-English bilingual.
- Version manifest.
- Resource rules and length policies.

`fixtures` contains only synthetic test resources. It must not contain official MapleLegends files.

## Translation Data

Use a human-editable manifest plus line-oriented translation data:

```text
translations/
  maplelegends/
    manifest.toml
    zh-CN/
      item.jsonl
      skill.jsonl
      map.jsonl
      quest-title.jsonl
      quest-info.jsonl
    zh-CN-bilingual/
      item.jsonl
      skill.jsonl
      map.jsonl
      quest-title.jsonl
      quest-info.jsonl
```

The manifest records:

- Game id.
- Supported client versions.
- Supported language modes.
- Resource identifiers.
- Local resource paths.
- Expected hashes.
- Resource kinds.
- Length policies.

Example manifest shape:

```toml
game = "maplelegends"
supported_client_versions = ["2026-xx"]
modes = ["zh-CN", "zh-CN-bilingual"]

[[resources]]
id = "string-item"
source = "String.wz/Item.img"
kind = "item"
hash = "..."
length_policy = "strict-name"

[[resources]]
id = "quest-info"
source = "Quest.wz/QuestInfo.img"
kind = "quest_info"
hash = "..."
length_policy = "expanded-text"
```

Translation entries use stable resource-derived keys:

```json
{"key":"item.2000000.name","source":"Red Potion","zh_CN":"红色药水","bilingual":"红色药水 / Red Potion","status":"reviewed"}
```

Rules:

- Keys are based on stable resource structure, not line numbers.
- `source` is for review and validation only.
- Bilingual text can be generated from the same entry unless manually overridden.
- Entries have review status.
- Validation reports missing, duplicate, stale, and over-length entries.
- The tool never silently truncates text.

## Length Policy

Length handling is resource-specific:

- UI labels, item names, map names, skill names, and quest titles use strict limits.
- Descriptions and quest info text can use expanded text limits where the resource format supports it.
- Any over-limit entry fails validation before patching.
- The app must report the exact entry and policy that failed.
- No automatic truncation, abbreviation, or heuristic rewrite is allowed.

## Install Flow

User-facing install flow:

```text
Select game directory
Validate MapleLegends executable and required resource files
Hash target resources
Match a known manifest version
Detect current patch status
Create backup snapshot
Build patch plan
Validate text policies
Write temporary patched files
Validate temporary outputs
Replace target files
Write installation record
Show result and restore entry point
```

Normal users cannot install against unknown hashes. Unknown versions show a clear unsupported-version message.

Developer mode is available through the CLI only. It can dry-run unknown versions and generate adaptation reports. It must not default to writing files.

## Restore Flow

Restore flow:

```text
Read installation record
Validate backup files exist
Validate backup hashes
Validate current files match this tool's installed outputs
Replace current files with backups
Mark restore complete
Show result
```

Backups and records live outside the game resource files, for example:

```text
%APPDATA%/OpenMaplePatch/installations/<client_hash>/
  install.json
  backups/
  logs/
```

Rules:

- Backups are mandatory before writing.
- Backup hash mismatch aborts restore.
- Install aborts if the game appears to be running.
- Restore aborts if the current files do not match a known installed state.
- Failures are explicit and logged.

## GUI

The GUI is a practical tool surface, not a landing page.

Primary screens:

- Install: choose directory, inspect status, choose language mode, install.
- Restore: inspect current patch status and restore original files.
- Translation Data: check and update translation data only.
- Logs: view and copy diagnostic information.
- Settings: language mode, backup location, developer logging.
- About: risk statement and project identity.

Interaction rules:

- Install is disabled until directory and manifest validation pass.
- Unknown versions are blocked for normal users.
- Before installing, show the files that will be changed.
- After installing, show backup location and restore action.
- Error messages must identify the failing check.
- The GUI calls `patch-core`; it does not implement file patching itself.

## Data Updates

The app can update translation data from the public project source.

Rules:

- Only translation data and manifests are updated automatically.
- Program binaries are not auto-updated.
- Program update notices may link to GitHub Releases.
- Data update must verify downloaded manifest integrity before use.
- Data update must not download or distribute MapleLegends client resources.

## Safety Boundaries

The tool must not:

- Read memory from the game process.
- Write memory to the game process.
- Inspect or modify packets.
- Automate input.
- Modify gameplay values.
- Modify map collision or traversal.
- Modify character appearance for other players.
- Ship original or modified MapleLegends resource files.
- Ship content copied from paid localization packages.
- Claim that the patch is officially allowed or risk-free.

The tool may:

- Read the user's selected local client files.
- Hash supported resource files.
- Apply text-only localization to known supported resources.
- Create backups and restore them.
- Generate reports for maintainers.

## Testing

Core tests:

- Manifest parsing.
- Translation entry parsing.
- Hash matching.
- Known-version acceptance.
- Unknown-version rejection.
- Developer dry-run behavior.
- Length policy validation.
- Patch plan generation.
- Backup creation.
- Restore validation.
- Failure rollback behavior.

Fixture tests:

- Use synthetic resource files only.
- Simulate install success.
- Simulate unsupported version.
- Simulate missing files.
- Simulate over-length translation.
- Simulate corrupted backup.
- Simulate restore success.

CLI tests:

- Validate translation data.
- Dry-run known version.
- Reject unknown version by default.
- Produce unknown-version report in developer mode.

GUI smoke tests:

- Directory selection.
- Status display.
- Install button disabled before validation.
- Install button enabled after validation.
- Restore state display.
- Log display.

Manual release checks:

- Verify on a clean Windows 10 VM.
- Verify on a clean Windows 11 VM.
- Use a user-provided local MapleLegends installation.
- Install localization.
- Start the game manually and inspect localized static text.
- Restore original files.
- Confirm restored files match backup hashes.

## Open Questions

The design intentionally leaves these decisions for implementation discovery:

- Exact resource parser/writer library or custom adapter.
- Exact manifest hash granularity for each resource.
- Exact format limits for each localized resource kind.
- Whether bilingual strings need per-resource formatting overrides.
- Whether the data update source is GitHub Releases, raw repository content, or a small static manifest endpoint.

These should be resolved by building against synthetic fixtures first, then validating against a user-provided local MapleLegends client.
