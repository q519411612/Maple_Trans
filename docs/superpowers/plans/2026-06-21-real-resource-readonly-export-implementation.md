# Real Resource Read-Only Export Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a maintainer-only `export-text` command that reads selected MapleLegends `.img` files from a local client directory and writes deterministic JSONL text exports outside the repository.

**Architecture:** Keep reusable behavior in `patch-core` and keep CLI concerns in `cli`. Automated tests use synthetic in-memory text trees and temporary directories; only manual verification touches the user-provided real client path, read-only.

**Tech Stack:** Rust 2021, `serde`, `serde_json`, `wz_reader` with default features disabled, `clap`, `tempfile`.

---

## File Structure

- Modify `Cargo.toml` to add `wz_reader` as a workspace dependency with default features disabled.
- Modify `patch-core/Cargo.toml` to depend on `wz_reader`.
- Create `patch-core/src/resource/export.rs` for export models, target resource definitions, directory validation, JSONL writing, and summary structs.
- Create `patch-core/src/resource/key.rs` for deterministic key generation from normalized resource paths and internal paths.
- Create `patch-core/src/resource/wz_img.rs` for the real `wz_reader` adapter.
- Modify `patch-core/src/resource/mod.rs` to expose the new modules.
- Create `patch-core/tests/export_tests.rs` for directory, key, JSONL, duplicate-key, and output behavior.
- Modify `cli/Cargo.toml` to add `serde_json` for CLI output tests if needed.
- Modify `cli/src/commands.rs` to add `export-text`.

## Chunk 1: Export Model And Key Rules

### Task 1: Define target resources and stable keys

**Files:**
- Create: `patch-core/src/resource/export.rs`
- Create: `patch-core/src/resource/key.rs`
- Modify: `patch-core/src/resource/mod.rs`
- Test: `patch-core/tests/export_tests.rs`

- [ ] **Step 1: Write failing tests**

Add tests that assert:

```rust
use patch_core::resource::export::{target_resources, ExportTextEntry};
use patch_core::resource::key::stable_text_key;

#[test]
fn target_resources_exclude_dialogue_say_resource() {
    let resources = target_resources();
    let paths: Vec<_> = resources.iter().map(|resource| resource.relative_path).collect();

    assert!(paths.contains(&"Data/String/Eqp.img"));
    assert!(paths.contains(&"Data/Quest/QuestInfo.img"));
    assert!(!paths.contains(&"Data/Quest/Say.img"));
}

#[test]
fn stable_text_key_uses_resource_kind_id_and_field() {
    let key = stable_text_key("Data/String/Eqp.img", "Eqp.img/01000001/name").unwrap();

    assert_eq!(key, "string.eqp.01000001.name");
}

#[test]
fn stable_text_key_rejects_unknown_field() {
    let error = stable_text_key("Data/String/Eqp.img", "Eqp.img/01000001/icon").unwrap_err();

    assert!(error.to_string().contains("unsupported text field"));
}

#[test]
fn export_text_entry_serializes_as_json_line() {
    let entry = ExportTextEntry {
        key: "string.eqp.01000001.name".to_owned(),
        source: "Sample text".to_owned(),
        resource: "Data/String/Eqp.img".to_owned(),
        path: "Eqp.img/01000001/name".to_owned(),
    };

    let line = entry.to_json_line().unwrap();

    assert_eq!(
        line,
        r#"{"key":"string.eqp.01000001.name","source":"Sample text","resource":"Data/String/Eqp.img","path":"Eqp.img/01000001/name"}"#
    );
}
```

- [ ] **Step 2: Run tests and verify they fail**

Run:

```bash
cargo test -p patch-core --test export_tests --locked
```

Expected: FAIL because the export modules do not exist yet.

- [ ] **Step 3: Implement minimal model and key rules**

Add:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TargetResource {
    pub id: &'static str,
    pub relative_path: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct ExportTextEntry {
    pub key: String,
    pub source: String,
    pub resource: String,
    pub path: String,
}
```

Implement `target_resources`, `ExportTextEntry::to_json_line`, and `stable_text_key`.

- [ ] **Step 4: Run tests and verify they pass**

Run:

```bash
cargo test -p patch-core --test export_tests --locked
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add patch-core/src/resource/export.rs patch-core/src/resource/key.rs patch-core/src/resource/mod.rs patch-core/tests/export_tests.rs
git commit -m "feat: add real resource export model"
```

## Chunk 2: Directory Validation And JSONL Writing

### Task 2: Validate client directory and write deterministic exports

**Files:**
- Modify: `patch-core/src/resource/export.rs`
- Modify: `patch-core/tests/export_tests.rs`

- [ ] **Step 1: Write failing tests**

Add tests that assert:

```rust
use std::fs;

use patch_core::resource::export::{
    validate_export_client_dir, write_export_jsonl, ExportTextEntry,
};

#[test]
fn validate_export_client_dir_requires_executable() {
    let temp = tempfile::tempdir().unwrap();

    let error = validate_export_client_dir(temp.path()).unwrap_err();

    assert!(error.to_string().contains("missing MapleLegends.exe"));
}

#[test]
fn validate_export_client_dir_requires_target_resources() {
    let temp = tempfile::tempdir().unwrap();
    fs::write(temp.path().join("MapleLegends.exe"), "").unwrap();

    let error = validate_export_client_dir(temp.path()).unwrap_err();

    assert!(error.to_string().contains("missing target resource"));
}

#[test]
fn write_export_jsonl_rejects_duplicate_keys() {
    let temp = tempfile::tempdir().unwrap();
    let entries = vec![
        ExportTextEntry {
            key: "string.eqp.01000001.name".to_owned(),
            source: "A".to_owned(),
            resource: "Data/String/Eqp.img".to_owned(),
            path: "Eqp.img/01000001/name".to_owned(),
        },
        ExportTextEntry {
            key: "string.eqp.01000001.name".to_owned(),
            source: "B".to_owned(),
            resource: "Data/String/Eqp.img".to_owned(),
            path: "Eqp.img/01000002/name".to_owned(),
        },
    ];

    let error = write_export_jsonl(temp.path(), "item.jsonl", &entries).unwrap_err();

    assert!(error.to_string().contains("duplicate export key"));
}

#[test]
fn write_export_jsonl_sorts_entries_by_key() {
    let temp = tempfile::tempdir().unwrap();
    let entries = vec![
        ExportTextEntry {
            key: "string.eqp.01000002.name".to_owned(),
            source: "B".to_owned(),
            resource: "Data/String/Eqp.img".to_owned(),
            path: "Eqp.img/01000002/name".to_owned(),
        },
        ExportTextEntry {
            key: "string.eqp.01000001.name".to_owned(),
            source: "A".to_owned(),
            resource: "Data/String/Eqp.img".to_owned(),
            path: "Eqp.img/01000001/name".to_owned(),
        },
    ];

    let path = write_export_jsonl(temp.path(), "item.jsonl", &entries).unwrap();
    let output = fs::read_to_string(path).unwrap();

    assert!(output.starts_with(r#"{"key":"string.eqp.01000001.name""#));
}
```

- [ ] **Step 2: Run tests and verify they fail**

Run:

```bash
cargo test -p patch-core --test export_tests --locked
```

Expected: FAIL because validation and writing functions do not exist.

- [ ] **Step 3: Implement minimal validation and writer**

Implement `validate_export_client_dir` and `write_export_jsonl`.
The validator checks only for `MapleLegends.exe` and required target resource paths.
The writer creates the output directory, rejects duplicate keys, sorts by key, writes one JSON object per line, and returns the written path.

- [ ] **Step 4: Run tests and verify they pass**

Run:

```bash
cargo test -p patch-core --test export_tests --locked
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add patch-core/src/resource/export.rs patch-core/tests/export_tests.rs
git commit -m "feat: add export validation and writer"
```

## Chunk 3: WZ IMG Adapter

### Task 3: Read text entries through `wz_reader`

**Files:**
- Modify: `Cargo.toml`
- Modify: `patch-core/Cargo.toml`
- Create: `patch-core/src/resource/wz_img.rs`
- Modify: `patch-core/src/resource/export.rs`
- Modify: `patch-core/src/resource/mod.rs`
- Test: `patch-core/tests/export_tests.rs`

- [ ] **Step 1: Write failing tests for adapter-independent behavior**

Add tests that call a pure collector function with synthetic path/value pairs:

```rust
use patch_core::resource::export::{collect_export_entries, RawTextNode};

#[test]
fn collect_export_entries_converts_known_text_nodes_to_entries() {
    let nodes = vec![RawTextNode {
        resource: "Data/String/Eqp.img".to_owned(),
        path: "Eqp.img/01000001/name".to_owned(),
        value: "Sample text".to_owned(),
    }];

    let entries = collect_export_entries(nodes).unwrap();

    assert_eq!(entries[0].key, "string.eqp.01000001.name");
    assert_eq!(entries[0].source, "Sample text");
}

#[test]
fn collect_export_entries_reports_unknown_text_nodes() {
    let nodes = vec![RawTextNode {
        resource: "Data/String/Eqp.img".to_owned(),
        path: "Eqp.img/01000001/icon".to_owned(),
        value: "icon data".to_owned(),
    }];

    let error = collect_export_entries(nodes).unwrap_err();

    assert!(error.to_string().contains("unsupported text field"));
}
```

- [ ] **Step 2: Run tests and verify they fail**

Run:

```bash
cargo test -p patch-core --test export_tests --locked
```

Expected: FAIL because `RawTextNode` and `collect_export_entries` do not exist.

- [ ] **Step 3: Implement pure collector and add dependency**

Add workspace dependency:

```toml
wz_reader = { version = "0.0.21", default-features = false }
```

Implement:

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RawTextNode {
    pub resource: String,
    pub path: String,
    pub value: String,
}
```

Add `collect_export_entries` to convert raw text nodes into `ExportTextEntry`.

- [ ] **Step 4: Implement real adapter behind tested collector**

In `wz_img.rs`, implement:

```rust
pub fn read_img_text_nodes(
    game_dir: &std::path::Path,
    target: TargetResource,
) -> PatchResult<Vec<RawTextNode>>
```

Use `WzNode::from_img_file(path, None, None)`, `walk_node(&node, true, ...)`, and `resolve_string_from_node`.
Collect only nodes that resolve as strings.
Do not write to the client directory.

- [ ] **Step 5: Run tests and verify they pass**

Run:

```bash
cargo test -p patch-core --test export_tests --locked
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml Cargo.lock patch-core/Cargo.toml patch-core/src/resource/export.rs patch-core/src/resource/wz_img.rs patch-core/src/resource/mod.rs patch-core/tests/export_tests.rs
git commit -m "feat: add read-only img text adapter"
```

## Chunk 4: CLI Command

### Task 4: Add `export-text`

**Files:**
- Modify: `cli/src/commands.rs`
- Test manually through CLI because the current CLI has no command integration test harness.

- [ ] **Step 1: Add a failing compile check expectation**

Run before implementation:

```bash
cargo run --locked -p omp-cli -- export-text --game-dir fixtures/synthetic-client --out /tmp/open-maple-export
```

Expected: FAIL because the subcommand does not exist.

- [ ] **Step 2: Implement command**

Add enum variant:

```rust
ExportText {
    #[arg(long)]
    game_dir: PathBuf,
    #[arg(long)]
    out: PathBuf,
}
```

Add function that validates the directory, reads target resources, writes one export file per target id, and prints a concise summary with resource count and entry count.

- [ ] **Step 3: Run command against an invalid directory**

Run:

```bash
cargo run --locked -p omp-cli -- export-text --game-dir fixtures/synthetic-client --out /tmp/open-maple-export
```

Expected: non-zero exit with a missing target resource message.

- [ ] **Step 4: Run full workspace verification**

Run:

```bash
cargo fmt --check
cargo test --workspace --locked
cargo clippy --workspace --locked -- -D warnings
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add cli/src/commands.rs
git commit -m "feat: add export text cli command"
```

## Chunk 5: Manual Read-Only Client Verification

### Task 5: Probe the user-provided MapleLegends client

**Files:**
- No source file edits expected.
- Output: `/Users/chenminghui/Documents/Codex/2026-06-21/g/work/maple-export`

- [ ] **Step 1: Record pre-run client state**

Run:

```bash
find '/Volumes/开发相关/MapleLegendsHD/Data/String' '/Volumes/开发相关/MapleLegendsHD/Data/Quest' -type f -name '*.img' -print0 | xargs -0 shasum -a 256 > /Users/chenminghui/Documents/Codex/2026-06-21/g/work/maple-client-before.sha256
```

- [ ] **Step 2: Run export**

Run:

```bash
cargo run --locked -p omp-cli -- export-text --game-dir '/Volumes/开发相关/MapleLegendsHD' --out /Users/chenminghui/Documents/Codex/2026-06-21/g/work/maple-export
```

Expected: command exits 0 and prints counts.

- [ ] **Step 3: Record post-run client state**

Run:

```bash
find '/Volumes/开发相关/MapleLegendsHD/Data/String' '/Volumes/开发相关/MapleLegendsHD/Data/Quest' -type f -name '*.img' -print0 | xargs -0 shasum -a 256 > /Users/chenminghui/Documents/Codex/2026-06-21/g/work/maple-client-after.sha256
diff -u /Users/chenminghui/Documents/Codex/2026-06-21/g/work/maple-client-before.sha256 /Users/chenminghui/Documents/Codex/2026-06-21/g/work/maple-client-after.sha256
```

Expected: no diff.

- [ ] **Step 4: Inspect generated output shape**

Run:

```bash
find /Users/chenminghui/Documents/Codex/2026-06-21/g/work/maple-export -maxdepth 1 -type f -print
head -n 3 /Users/chenminghui/Documents/Codex/2026-06-21/g/work/maple-export/*.jsonl
```

Expected: JSONL files exist outside the repository and contain stable keys.

- [ ] **Step 5: Do not commit generated exports**

Run:

```bash
git status --short
```

Expected: generated export files do not appear in repository status.

## Final Verification

Run:

```bash
cargo fmt --check
cargo test --workspace --locked
cargo clippy --workspace --locked -- -D warnings
cargo run --locked -p omp-cli -- validate-data --manifest translations/maplelegends/manifest.toml --translations translations/maplelegends/zh-CN/item.jsonl
cargo run --locked -p omp-cli -- validate-data --manifest translations/maplelegends/manifest.toml --translations translations/maplelegends/zh-CN-bilingual/item.jsonl
```

Expected: all commands pass.

Run repository hygiene checks:

```bash
find . -type f \( -name '*.wz' -o -name '*.img' -o -name '*.exe' \) -print
git diff --name-only HEAD
```

Expected: no client resources in the repository, and only intended source/docs changes.
