# Real Resource Read-Only Export Design

## Purpose

This design adds the first real MapleLegends resource adapter to Open Maple Patch.
The adapter reads text from a user-selected local MapleLegends installation and produces maintainer diagnostics outside the repository.

This scope is intentionally read-only. It does not patch, rewrite, replace, package, or distribute MapleLegends client resources.

## Public Positioning

All public wording for this capability must stay aligned with the project boundary:

- Unofficial.
- Use at your own risk.
- Text-only localization research.
- No gameplay advantage.
- No affiliation with MapleLegends or Nexon.
- No distribution of original or modified client resources.
- No claim that the patch is officially allowed, safe, or risk-free.

## Confirmed Resource Layout

The user-provided client directory uses an expanded data layout:

```text
MapleLegendsHD/
  MapleLegends.exe
  Data/
    String/
      Eqp.img
      Consume.img
      Etc.img
      Ins.img
      Cash.img
      Pet.img
      Skill.img
      Map.img
    Quest/
      QuestInfo.img
      Say.img
```

The first real adapter targets standalone `.img` files under `Data/String` and `Data/Quest`.
It does not assume root-level `String.wz` or `Quest.wz` files.

## Target Resources

The first read-only export covers these static text resources:

```text
Data/String/Eqp.img
Data/String/Consume.img
Data/String/Etc.img
Data/String/Ins.img
Data/String/Cash.img
Data/String/Pet.img
Data/String/Skill.img
Data/String/Map.img
Data/Quest/QuestInfo.img
```

`Data/Quest/Say.img` is excluded from patch targets in this slice. It may be probed later as a separate conversation because it can include NPC dialogue or story text, which has a different risk and scope profile.

## Non-Goals

This work does not:

- Write `.img` or `.wz` files.
- Modify the real client directory.
- Create backups or installation records for real resources.
- Add GUI install behavior for real resources.
- Include extracted text dumps in the repository.
- Use paid localization data as a source.
- Patch NPC dialogue, story text, gameplay values, packets, memory, automation behavior, or executable files.

## Proposed Approach

The project should add a real-resource read adapter in `patch-core` and expose it through a maintainer CLI command.

Recommended command shape:

```text
omp-cli export-text --game-dir <MapleLegendsHD> --out <output-dir>
```

The command:

- Validates that `MapleLegends.exe` exists.
- Validates that every target `.img` file exists.
- Parses target `.img` files through a Rust WZ/IMG library with no image decoding features enabled.
- Walks only text-bearing nodes in the allowed target resources.
- Writes JSONL export files to the requested output directory.
- Prints counts, resource hashes, and sample keys.

Generated exports are local maintainer artifacts. They are not committed.

## Data Model

Each exported text entry should be explicit and stable:

```json
{
  "key": "string.eqp.<item_id>.name",
  "source": "<source text>",
  "resource": "Data/String/Eqp.img",
  "path": "Eqp.img/<item_id>/name"
}
```

Fields:

- `key`: stable project translation key.
- `source`: original text found in the local client.
- `resource`: normalized resource path relative to the client root.
- `path`: resource-internal path used for diagnostics.

The export should preserve enough path information for maintainers to trace a bad key back to the local resource structure without storing the original resource file.

## Key Rules

Keys are derived from resource identity and resource-internal structure, not from row numbers or traversal order.

Initial key prefixes:

```text
string.eqp.<id>.<field>
string.consume.<id>.<field>
string.etc.<id>.<field>
string.ins.<id>.<field>
string.cash.<id>.<field>
string.pet.<id>.<field>
string.skill.<id>.<field>
string.map.<id>.<field>
quest.info.<quest_id>.<field>
```

The adapter should collect known text fields such as `name`, `desc`, `h`, `mapName`, `streetName`, and quest title or description fields after they are confirmed by real traversal output.

Unknown text fields are reported separately instead of silently folded into translation data. This keeps key generation auditable and avoids heuristic post-processing.

## Error Handling

The command fails loudly when:

- The selected directory is not a MapleLegends client directory.
- A required target resource is missing.
- A target resource cannot be parsed.
- The output directory cannot be created or written.
- Duplicate stable keys are generated.
- A text value cannot be represented in UTF-8.

The command may continue across resources only when explicitly asked by a future diagnostic flag. The default path should fail early so parser or key-shape problems are visible.

## Library Selection

The first implementation should try `wz_reader` with default features disabled so text extraction does not pull native image or compression build dependencies unnecessarily.

The implementation should prefer reading standalone `.img` files directly. If the library cannot support this target layout cleanly, the implementation plan must stop and document the blocker before adding custom parsing logic.

`wzlib-rs` remains a candidate for future write support, but this design does not require write support.

## Integration Boundaries

`patch-core` owns:

- Target resource definitions.
- Client directory validation.
- `.img` parsing adapter.
- Text extraction.
- Stable key generation.
- JSONL export model.

`cli` owns:

- Command arguments.
- Human-readable summary output.
- Exit status.

The GUI should not call this feature in the initial implementation. This keeps real-resource exploration in maintainer tooling until parsing and key rules are verified.

## Verification

Automated verification uses synthetic fixtures only:

- Directory validation rejects missing executable or target resources.
- Export model serializes deterministic JSONL.
- Key generation rejects duplicates.
- Unknown fields are reported separately.
- CLI exits non-zero for invalid client directories.

Manual verification uses the user-provided local client directory read-only:

```text
omp-cli export-text \
  --game-dir /Volumes/开发相关/MapleLegendsHD \
  --out /Users/chenminghui/Documents/Codex/2026-06-21/g/work/maple-export
```

Manual verification must confirm:

- No file under the client directory changes.
- Output files are written only under the requested output directory.
- Target resource hashes are printed.
- Counts and sample keys are stable across repeated runs.

## Open Decisions

These decisions should be made after the first read-only traversal succeeds:

- Final field allow-list per resource kind.
- Whether map, skill, and quest descriptions need separate length policies.
- Whether bilingual data should be generated from one entry or stored explicitly per resource kind.
- Whether `Data/Quest/Say.img` should remain excluded permanently or become an opt-in diagnostic-only export.

## Manual Probe Finding

The first read-only probe against the user-provided MapleLegendsHD client reached the standalone `.img` parser but did not complete.

Observed behavior:

- `wz_reader` can open the target file path but cannot auto-detect the image version.
- Explicit `GMS`, `EMS`, and `BMS` version attempts all fail with a wrong-version error.
- Fixed IV attempts using zero, GMS, and MSEA IVs also fail.
- `wzlib-rs` can parse `list.wz` with the GMS IV and returns readable paths, but its hotfix image parser also does not parse `Data/String/Eqp.img` with known IVs.
- Client file hashes before and after the failed probe match exactly.

Current conclusion:

MapleLegendsHD standalone `.img` files appear to require MapleLegends-specific encryption parameters or a compatible parser beyond the default public `GMS`, `EMS`, and `BMS` IV modes. The project should not guess or brute-force this. The next resource-adaptation slice needs one of:

- A documented compatible open-source parser path.
- The correct non-secret parser configuration if it is safe to publish.
- A user-generated local text export that can be used to start translation data without distributing client resources.
