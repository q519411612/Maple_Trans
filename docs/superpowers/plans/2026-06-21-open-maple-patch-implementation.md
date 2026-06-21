# Open Maple Patch Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust + Tauri open-source MapleLegends text localization assistant that can validate translation data, patch synthetic local resources, create backups, restore originals, and expose the flow through CLI and GUI without shipping any MapleLegends client files.

**Architecture:** Use a monorepo with `patch-core` as the only crate that owns file mutation, `cli` as the maintainer/developer surface, `tauri-app` as the user GUI, `translations` as public data, and `fixtures` as synthetic test resources. Real MapleLegends resource support is intentionally isolated behind a `ResourceCodec` interface so the first implementation can be verified safely with fake resources.

**Tech Stack:** Rust workspace, Cargo, Serde, TOML, JSONL, SHA-256 hashing, tempfile, thiserror, clap, Tauri, TypeScript, Vitest.

---

## Preconditions

- This directory is currently not a git repository. Before execution, either create a repository here with `git init` or move this plan into an existing repository. Commit checkpoints below assume git is available.
- Do not add MapleLegends client files, modified resources, extracted resource dumps, paid localization files, or screenshots of patched gameplay to the repository.
- Real client support must not be implemented by guessing, bypassing, or publishing protected format details. Use synthetic fixtures first.

## File Structure

Create this structure:

```text
Cargo.toml
README.md
.gitignore
patch-core/
  Cargo.toml
  src/
    lib.rs
    error.rs
    manifest.rs
    translation.rs
    hash.rs
    length.rs
    resource/
      mod.rs
      synthetic.rs
    plan.rs
    backup.rs
    install.rs
    restore.rs
    report.rs
  tests/
    manifest_tests.rs
    translation_tests.rs
    length_tests.rs
    patch_plan_tests.rs
    install_restore_tests.rs
cli/
  Cargo.toml
  src/
    main.rs
    commands.rs
tauri-app/
  package.json
  src/
    App.tsx
    main.tsx
    api.ts
    styles.css
  src-tauri/
    Cargo.toml
    tauri.conf.json
    src/
      main.rs
      commands.rs
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
fixtures/
  synthetic-client/
    MapleLegends.exe.placeholder
    String/
      Item.synthetic.json
      Skill.synthetic.json
      Map.synthetic.json
    Quest/
      QuestInfo.synthetic.json
  expected/
    patched-item.synthetic.json
docs/
  superpowers/
    specs/
    plans/
```

## Chunk 1: Workspace And Project Guardrails

### Task 1: Create Rust workspace skeleton

**Files:**
- Create: `Cargo.toml`
- Create: `patch-core/Cargo.toml`
- Create: `patch-core/src/lib.rs`
- Create: `patch-core/src/error.rs`
- Create: `cli/Cargo.toml`
- Create: `cli/src/main.rs`
- Create: `.gitignore`

- [ ] **Step 1: Write workspace manifests**

`Cargo.toml`:

```toml
[workspace]
members = ["patch-core", "cli"]
resolver = "2"

[workspace.package]
edition = "2021"
license = "MIT OR Apache-2.0"
repository = "https://github.com/OWNER/open-maple-patch"

[workspace.dependencies]
anyhow = "1"
clap = { version = "4", features = ["derive"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
sha2 = "0.10"
tempfile = "3"
thiserror = "1"
toml = "0.8"
```

`patch-core/Cargo.toml`:

```toml
[package]
name = "patch-core"
version = "0.1.0"
edition.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
serde.workspace = true
serde_json.workspace = true
sha2.workspace = true
thiserror.workspace = true
toml.workspace = true

[dev-dependencies]
tempfile.workspace = true
```

`cli/Cargo.toml`:

```toml
[package]
name = "omp-cli"
version = "0.1.0"
edition.workspace = true
license.workspace = true
repository.workspace = true

[dependencies]
anyhow.workspace = true
clap.workspace = true
patch-core = { path = "../patch-core" }
```

`.gitignore`:

```gitignore
/target/
/tauri-app/node_modules/
/tauri-app/dist/
/tauri-app/src-tauri/target/
.superpowers/
*.log
```

- [ ] **Step 2: Add minimal Rust entry points**

`patch-core/src/lib.rs`:

```rust
pub mod error;

pub use error::{PatchError, PatchResult};
```

`patch-core/src/error.rs`:

```rust
use thiserror::Error;

pub type PatchResult<T> = Result<T, PatchError>;

#[derive(Debug, Error)]
pub enum PatchError {
    #[error("unsupported client version: {0}")]
    UnsupportedClientVersion(String),
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
```

`cli/src/main.rs`:

```rust
fn main() {
    println!("open-maple-patch cli");
}
```

- [ ] **Step 3: Run workspace build**

Run: `cargo test --workspace`

Expected: all crates compile and tests report `0 passed`.

- [ ] **Step 4: Commit**

```bash
git add Cargo.toml patch-core cli .gitignore
git commit -m "chore: scaffold rust workspace"
```

### Task 2: Add README safety positioning

**Files:**
- Create: `README.md`

- [ ] **Step 1: Write README baseline**

Include:

```markdown
# Open Maple Patch

Open Maple Patch is an unofficial text localization patcher for MapleLegends.

This project is not affiliated with MapleLegends or Nexon.

This project does not distribute MapleLegends client files.
This project does not modify gameplay values, map collision, character state, packets, memory, or automation behavior.
Use at your own risk.

The repository contains code, translation data, manifests, and synthetic test fixtures only.
Do not upload original or modified MapleLegends resource files.
```

- [ ] **Step 2: Check README for risky claims**

Run: `rg -n "officially allowed|risk-free|不会封号|零风险|guaranteed" README.md`

Expected: no matches.

- [ ] **Step 3: Commit**

```bash
git add README.md
git commit -m "docs: add project safety statement"
```

## Chunk 2: Manifest And Translation Data Validation

### Task 3: Parse manifest files

**Files:**
- Create: `patch-core/src/manifest.rs`
- Modify: `patch-core/src/lib.rs`
- Test: `patch-core/tests/manifest_tests.rs`

- [ ] **Step 1: Write failing manifest tests**

`patch-core/tests/manifest_tests.rs`:

```rust
use patch_core::manifest::{LengthPolicy, Manifest, ResourceKind};

#[test]
fn parses_manifest_with_supported_modes_and_resources() {
    let input = r#"
game = "maplelegends"
supported_client_versions = ["synthetic-001"]
modes = ["zh-CN", "zh-CN-bilingual"]

[[resources]]
id = "string-item"
source = "String/Item.synthetic.json"
kind = "item"
hash = "abc123"
length_policy = "strict-name"
"#;

    let manifest: Manifest = toml::from_str(input).unwrap();

    assert_eq!(manifest.game, "maplelegends");
    assert_eq!(manifest.supported_client_versions, vec!["synthetic-001"]);
    assert_eq!(manifest.modes, vec!["zh-CN", "zh-CN-bilingual"]);
    assert_eq!(manifest.resources[0].kind, ResourceKind::Item);
    assert_eq!(manifest.resources[0].length_policy, LengthPolicy::StrictName);
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test -p patch-core --test manifest_tests`

Expected: compile fails because `manifest` module does not exist.

- [ ] **Step 3: Implement manifest model**

`patch-core/src/manifest.rs`:

```rust
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct Manifest {
    pub game: String,
    pub supported_client_versions: Vec<String>,
    pub modes: Vec<String>,
    pub resources: Vec<ResourceManifest>,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct ResourceManifest {
    pub id: String,
    pub source: String,
    pub kind: ResourceKind,
    pub hash: String,
    pub length_policy: LengthPolicy,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ResourceKind {
    Item,
    Skill,
    Map,
    QuestTitle,
    QuestInfo,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum LengthPolicy {
    StrictName,
    ExpandedText,
}
```

`patch-core/src/lib.rs`:

```rust
pub mod error;
pub mod manifest;

pub use error::{PatchError, PatchResult};
```

- [ ] **Step 4: Run test to verify pass**

Run: `cargo test -p patch-core --test manifest_tests`

Expected: test passes.

- [ ] **Step 5: Commit**

```bash
git add patch-core/src/lib.rs patch-core/src/manifest.rs patch-core/tests/manifest_tests.rs
git commit -m "feat: parse localization manifest"
```

### Task 4: Parse JSONL translation entries

**Files:**
- Create: `patch-core/src/translation.rs`
- Modify: `patch-core/src/lib.rs`
- Test: `patch-core/tests/translation_tests.rs`

- [ ] **Step 1: Write failing translation tests**

`patch-core/tests/translation_tests.rs`:

```rust
use patch_core::translation::{parse_jsonl, ReviewStatus};

#[test]
fn parses_translation_jsonl_entries() {
    let input = r#"{"key":"item.2000000.name","source":"Red Potion","zh_CN":"红色药水","bilingual":"红色药水 / Red Potion","status":"reviewed"}
"#;

    let entries = parse_jsonl(input).unwrap();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].key, "item.2000000.name");
    assert_eq!(entries[0].zh_cn, "红色药水");
    assert_eq!(entries[0].status, ReviewStatus::Reviewed);
}

#[test]
fn rejects_duplicate_keys() {
    let input = r#"{"key":"item.1.name","source":"A","zh_CN":"甲","bilingual":"甲 / A","status":"draft"}
{"key":"item.1.name","source":"A","zh_CN":"乙","bilingual":"乙 / A","status":"draft"}
"#;

    let error = parse_jsonl(input).unwrap_err().to_string();
    assert!(error.contains("duplicate translation key"));
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test -p patch-core --test translation_tests`

Expected: compile fails because `translation` module does not exist.

- [ ] **Step 3: Implement translation parser**

`patch-core/src/translation.rs`:

```rust
use std::collections::HashSet;

use serde::Deserialize;

use crate::error::{PatchError, PatchResult};

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
pub struct TranslationEntry {
    pub key: String,
    pub source: String,
    #[serde(rename = "zh_CN")]
    pub zh_cn: String,
    pub bilingual: String,
    pub status: ReviewStatus,
}

#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ReviewStatus {
    Draft,
    Reviewed,
}

pub fn parse_jsonl(input: &str) -> PatchResult<Vec<TranslationEntry>> {
    let mut entries = Vec::new();
    let mut keys = HashSet::new();

    for (index, line) in input.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let entry: TranslationEntry = serde_json::from_str(line)
            .map_err(|error| PatchError::Validation(format!("invalid jsonl line {}: {}", index + 1, error)))?;
        if !keys.insert(entry.key.clone()) {
            return Err(PatchError::Validation(format!("duplicate translation key: {}", entry.key)));
        }
        entries.push(entry);
    }

    Ok(entries)
}
```

Update `patch-core/src/lib.rs`:

```rust
pub mod error;
pub mod manifest;
pub mod translation;

pub use error::{PatchError, PatchResult};
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p patch-core --test translation_tests`

Expected: tests pass.

- [ ] **Step 5: Commit**

```bash
git add patch-core/src/lib.rs patch-core/src/translation.rs patch-core/tests/translation_tests.rs
git commit -m "feat: parse translation jsonl"
```

### Task 5: Validate length policies

**Files:**
- Create: `patch-core/src/length.rs`
- Modify: `patch-core/src/lib.rs`
- Test: `patch-core/tests/length_tests.rs`

- [ ] **Step 1: Write failing length tests**

`patch-core/tests/length_tests.rs`:

```rust
use patch_core::length::{validate_text_length, TextTarget};
use patch_core::manifest::LengthPolicy;

#[test]
fn accepts_short_strict_name() {
    validate_text_length(
        TextTarget { key: "item.1.name", text: "红色药水" },
        LengthPolicy::StrictName,
    )
    .unwrap();
}

#[test]
fn rejects_overlong_strict_name() {
    let error = validate_text_length(
        TextTarget { key: "item.1.name", text: "这是一段明显超过首版严格名称预算的物品名称" },
        LengthPolicy::StrictName,
    )
    .unwrap_err()
    .to_string();

    assert!(error.contains("item.1.name"));
    assert!(error.contains("strict-name"));
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test -p patch-core --test length_tests`

Expected: compile fails because `length` module does not exist.

- [ ] **Step 3: Implement initial policy**

`patch-core/src/length.rs`:

```rust
use crate::error::{PatchError, PatchResult};
use crate::manifest::LengthPolicy;

pub struct TextTarget<'a> {
    pub key: &'a str,
    pub text: &'a str,
}

pub fn validate_text_length(target: TextTarget<'_>, policy: LengthPolicy) -> PatchResult<()> {
    let max_chars = match policy {
        LengthPolicy::StrictName => 16,
        LengthPolicy::ExpandedText => 512,
    };

    let actual = target.text.chars().count();
    if actual > max_chars {
        return Err(PatchError::Validation(format!(
            "text length exceeds {:?} for {}: {} > {}",
            policy, target.key, actual, max_chars
        )));
    }

    Ok(())
}
```

Update `patch-core/src/lib.rs`:

```rust
pub mod error;
pub mod length;
pub mod manifest;
pub mod translation;

pub use error::{PatchError, PatchResult};
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p patch-core --test length_tests`

Expected: tests pass.

- [ ] **Step 5: Commit**

```bash
git add patch-core/src/lib.rs patch-core/src/length.rs patch-core/tests/length_tests.rs
git commit -m "feat: validate text length policies"
```

## Chunk 3: Synthetic Resource Codec And Patch Planning

### Task 6: Add synthetic resource codec

**Files:**
- Create: `patch-core/src/resource/mod.rs`
- Create: `patch-core/src/resource/synthetic.rs`
- Modify: `patch-core/src/lib.rs`
- Test: `patch-core/tests/patch_plan_tests.rs`

- [ ] **Step 1: Write failing codec test**

`patch-core/tests/patch_plan_tests.rs`:

```rust
use patch_core::resource::synthetic::SyntheticResource;

#[test]
fn reads_and_writes_synthetic_resource_text() {
    let input = r#"{"item.2000000.name":"Red Potion"}"#;
    let mut resource = SyntheticResource::from_str(input).unwrap();

    resource.set_text("item.2000000.name", "红色药水").unwrap();
    let output = resource.to_string_pretty().unwrap();

    assert!(output.contains("红色药水"));
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test -p patch-core --test patch_plan_tests`

Expected: compile fails because `resource` module does not exist.

- [ ] **Step 3: Implement synthetic codec**

`patch-core/src/resource/mod.rs`:

```rust
use crate::error::PatchResult;

pub mod synthetic;

pub trait ResourceText {
    fn set_text(&mut self, key: &str, value: &str) -> PatchResult<()>;
}
```

`patch-core/src/resource/synthetic.rs`:

```rust
use std::collections::BTreeMap;

use crate::error::{PatchError, PatchResult};
use crate::resource::ResourceText;

#[derive(Debug, Clone)]
pub struct SyntheticResource {
    values: BTreeMap<String, String>,
}

impl SyntheticResource {
    pub fn from_str(input: &str) -> PatchResult<Self> {
        let values = serde_json::from_str(input)
            .map_err(|error| PatchError::Validation(format!("invalid synthetic resource: {}", error)))?;
        Ok(Self { values })
    }

    pub fn to_string_pretty(&self) -> PatchResult<String> {
        serde_json::to_string_pretty(&self.values)
            .map_err(|error| PatchError::Validation(format!("cannot write synthetic resource: {}", error)))
    }
}

impl ResourceText for SyntheticResource {
    fn set_text(&mut self, key: &str, value: &str) -> PatchResult<()> {
        let slot = self
            .values
            .get_mut(key)
            .ok_or_else(|| PatchError::Validation(format!("missing resource key: {}", key)))?;
        *slot = value.to_owned();
        Ok(())
    }
}
```

Update `patch-core/src/lib.rs`:

```rust
pub mod error;
pub mod length;
pub mod manifest;
pub mod resource;
pub mod translation;

pub use error::{PatchError, PatchResult};
```

- [ ] **Step 4: Run test**

Run: `cargo test -p patch-core --test patch_plan_tests`

Expected: test passes.

- [ ] **Step 5: Commit**

```bash
git add patch-core/src/lib.rs patch-core/src/resource patch-core/tests/patch_plan_tests.rs
git commit -m "feat: add synthetic resource codec"
```

### Task 7: Build patch plans without writing files

**Files:**
- Create: `patch-core/src/plan.rs`
- Modify: `patch-core/src/lib.rs`
- Modify: `patch-core/tests/patch_plan_tests.rs`

- [ ] **Step 1: Add failing patch plan test**

Add to `patch-core/tests/patch_plan_tests.rs`:

```rust
use patch_core::manifest::{LengthPolicy, ResourceKind, ResourceManifest};
use patch_core::plan::{build_patch_plan, LanguageMode};
use patch_core::translation::{ReviewStatus, TranslationEntry};

#[test]
fn builds_patch_plan_for_reviewed_entries() {
    let resource = ResourceManifest {
        id: "string-item".to_owned(),
        source: "String/Item.synthetic.json".to_owned(),
        kind: ResourceKind::Item,
        hash: "abc123".to_owned(),
        length_policy: LengthPolicy::StrictName,
    };
    let entries = vec![TranslationEntry {
        key: "item.2000000.name".to_owned(),
        source: "Red Potion".to_owned(),
        zh_cn: "红色药水".to_owned(),
        bilingual: "红色药水 / Red Potion".to_owned(),
        status: ReviewStatus::Reviewed,
    }];

    let plan = build_patch_plan(&resource, &entries, LanguageMode::SimplifiedChinese).unwrap();

    assert_eq!(plan.edits.len(), 1);
    assert_eq!(plan.edits[0].replacement, "红色药水");
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test -p patch-core --test patch_plan_tests`

Expected: compile fails because `plan` module does not exist.

- [ ] **Step 3: Implement patch planning**

`patch-core/src/plan.rs`:

```rust
use crate::error::PatchResult;
use crate::length::{validate_text_length, TextTarget};
use crate::manifest::ResourceManifest;
use crate::translation::{ReviewStatus, TranslationEntry};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LanguageMode {
    SimplifiedChinese,
    Bilingual,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PatchPlan {
    pub resource_id: String,
    pub source: String,
    pub edits: Vec<TextEdit>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextEdit {
    pub key: String,
    pub replacement: String,
}

pub fn build_patch_plan(
    resource: &ResourceManifest,
    entries: &[TranslationEntry],
    mode: LanguageMode,
) -> PatchResult<PatchPlan> {
    let mut edits = Vec::new();

    for entry in entries {
        if entry.status != ReviewStatus::Reviewed {
            continue;
        }
        let replacement = match mode {
            LanguageMode::SimplifiedChinese => entry.zh_cn.clone(),
            LanguageMode::Bilingual => entry.bilingual.clone(),
        };
        validate_text_length(
            TextTarget { key: &entry.key, text: &replacement },
            resource.length_policy.clone(),
        )?;
        edits.push(TextEdit { key: entry.key.clone(), replacement });
    }

    Ok(PatchPlan {
        resource_id: resource.id.clone(),
        source: resource.source.clone(),
        edits,
    })
}
```

Update `patch-core/src/lib.rs`:

```rust
pub mod error;
pub mod length;
pub mod manifest;
pub mod plan;
pub mod resource;
pub mod translation;

pub use error::{PatchError, PatchResult};
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p patch-core --test patch_plan_tests`

Expected: all patch plan tests pass.

- [ ] **Step 5: Commit**

```bash
git add patch-core/src/lib.rs patch-core/src/plan.rs patch-core/tests/patch_plan_tests.rs
git commit -m "feat: build text patch plans"
```

## Chunk 4: Hashing, Backup, Install, Restore

### Task 8: Add file hashing

**Files:**
- Create: `patch-core/src/hash.rs`
- Modify: `patch-core/src/lib.rs`
- Test: `patch-core/tests/install_restore_tests.rs`

- [ ] **Step 1: Write failing hash test**

`patch-core/tests/install_restore_tests.rs`:

```rust
use std::fs;

use patch_core::hash::sha256_file_hex;

#[test]
fn computes_file_sha256() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("sample.txt");
    fs::write(&path, "abc").unwrap();

    let hash = sha256_file_hex(&path).unwrap();

    assert_eq!(hash, "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test -p patch-core --test install_restore_tests`

Expected: compile fails because `hash` module does not exist.

- [ ] **Step 3: Implement hash helper**

`patch-core/src/hash.rs`:

```rust
use std::fs::File;
use std::io::Read;
use std::path::Path;

use sha2::{Digest, Sha256};

use crate::error::PatchResult;

pub fn sha256_file_hex(path: &Path) -> PatchResult<String> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];

    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}
```

Update `patch-core/src/lib.rs`.

- [ ] **Step 4: Run test**

Run: `cargo test -p patch-core --test install_restore_tests`

Expected: hash test passes.

- [ ] **Step 5: Commit**

```bash
git add patch-core/src/lib.rs patch-core/src/hash.rs patch-core/tests/install_restore_tests.rs
git commit -m "feat: hash local resource files"
```

### Task 9: Implement backup and restore records

**Files:**
- Create: `patch-core/src/backup.rs`
- Create: `patch-core/src/restore.rs`
- Modify: `patch-core/src/lib.rs`
- Modify: `patch-core/tests/install_restore_tests.rs`

- [ ] **Step 1: Add failing backup/restore test**

Add to `patch-core/tests/install_restore_tests.rs`:

```rust
use patch_core::backup::{create_backup, BackupRequest};
use patch_core::restore::{restore_backup, RestoreRequest};

#[test]
fn creates_backup_and_restores_original_file() {
    let dir = tempfile::tempdir().unwrap();
    let game_dir = dir.path().join("game");
    let backup_dir = dir.path().join("backup");
    fs::create_dir_all(&game_dir).unwrap();

    let resource = game_dir.join("String/Item.synthetic.json");
    fs::create_dir_all(resource.parent().unwrap()).unwrap();
    fs::write(&resource, r#"{"item.1.name":"Red Potion"}"#).unwrap();

    let record = create_backup(BackupRequest {
        game_dir: &game_dir,
        backup_dir: &backup_dir,
        relative_files: &["String/Item.synthetic.json"],
    })
    .unwrap();

    fs::write(&resource, r#"{"item.1.name":"红色药水"}"#).unwrap();

    restore_backup(RestoreRequest {
        game_dir: &game_dir,
        backup_dir: &backup_dir,
        record: &record,
    })
    .unwrap();

    let restored = fs::read_to_string(&resource).unwrap();
    assert!(restored.contains("Red Potion"));
}
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test -p patch-core --test install_restore_tests`

Expected: compile fails because backup and restore modules do not exist.

- [ ] **Step 3: Implement backup and restore**

Create minimal implementations that:

- Preserve relative paths inside backup directory.
- Store original file hashes.
- Restore only files listed in the backup record.
- Return an explicit validation error if a backup file is missing.

- [ ] **Step 4: Run tests**

Run: `cargo test -p patch-core --test install_restore_tests`

Expected: backup/restore test passes.

- [ ] **Step 5: Commit**

```bash
git add patch-core/src/lib.rs patch-core/src/backup.rs patch-core/src/restore.rs patch-core/tests/install_restore_tests.rs
git commit -m "feat: backup and restore resources"
```

### Task 10: Implement synthetic install engine

**Files:**
- Create: `patch-core/src/install.rs`
- Modify: `patch-core/src/lib.rs`
- Modify: `patch-core/tests/install_restore_tests.rs`

- [ ] **Step 1: Add failing install test**

Add a test that:

- Creates a temp synthetic client.
- Creates a manifest hash for the synthetic resource.
- Builds one reviewed translation entry.
- Runs install.
- Verifies the resource text changed.
- Verifies backup exists.

Expected assertion:

```rust
assert!(patched.contains("红色药水"));
assert!(backup_dir.join("String/Item.synthetic.json").exists());
```

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test -p patch-core --test install_restore_tests`

Expected: compile fails because `install` module does not exist.

- [ ] **Step 3: Implement synthetic install**

Implement an `InstallRequest` with:

- `game_dir`
- `backup_dir`
- `manifest`
- `resource`
- `entries`
- `language_mode`
- `allow_unknown_version`

Rules:

- If the actual hash does not match manifest and `allow_unknown_version` is false, return `UnsupportedClientVersion`.
- Always create backup before writing.
- Write to a temp sibling file first.
- Replace target after synthetic output is generated.

- [ ] **Step 4: Run tests**

Run: `cargo test -p patch-core --test install_restore_tests`

Expected: all install/restore tests pass.

- [ ] **Step 5: Commit**

```bash
git add patch-core/src/lib.rs patch-core/src/install.rs patch-core/tests/install_restore_tests.rs
git commit -m "feat: install synthetic text patch"
```

## Chunk 5: Public Translation Data And CLI

### Task 11: Add synthetic translation dataset

**Files:**
- Create: `translations/maplelegends/manifest.toml`
- Create: `translations/maplelegends/zh-CN/item.jsonl`
- Create: `translations/maplelegends/zh-CN-bilingual/item.jsonl`
- Create: `fixtures/synthetic-client/MapleLegends.exe.placeholder`
- Create: `fixtures/synthetic-client/String/Item.synthetic.json`

- [ ] **Step 1: Add synthetic fixture files**

Use fake resource content only:

```json
{
  "item.2000000.name": "Red Potion"
}
```

Use fake translation entries:

```json
{"key":"item.2000000.name","source":"Red Potion","zh_CN":"红色药水","bilingual":"红色药水 / Red Potion","status":"reviewed"}
```

- [ ] **Step 2: Compute fixture hash**

Run: `shasum -a 256 fixtures/synthetic-client/String/Item.synthetic.json`

Expected: one hash line. Insert that hash in `translations/maplelegends/manifest.toml`.

- [ ] **Step 3: Validate no official files were added**

Run: `find fixtures translations -type f | rg -n "wz|img|exe$|MapleLegends\\.exe$"`

Expected: no matches except `MapleLegends.exe.placeholder`.

- [ ] **Step 4: Commit**

```bash
git add translations fixtures
git commit -m "test: add synthetic localization fixtures"
```

### Task 12: Implement CLI validate and dry-run commands

**Files:**
- Create: `cli/src/commands.rs`
- Modify: `cli/src/main.rs`

- [ ] **Step 1: Write command behavior**

Commands:

```text
omp-cli validate-data --manifest translations/maplelegends/manifest.toml --translations translations/maplelegends/zh-CN/item.jsonl
omp-cli dry-run --game-dir fixtures/synthetic-client --manifest translations/maplelegends/manifest.toml --translations translations/maplelegends/zh-CN/item.jsonl
```

- [ ] **Step 2: Run command to verify current failure**

Run: `cargo run -p omp-cli -- validate-data --manifest translations/maplelegends/manifest.toml --translations translations/maplelegends/zh-CN/item.jsonl`

Expected: fails because command is not implemented.

- [ ] **Step 3: Implement clap commands**

`cli/src/main.rs` should parse subcommands and delegate to `commands.rs`.

`validate-data` should:

- Parse manifest.
- Parse translation JSONL.
- Validate lengths for referenced resource policy.
- Print `validation ok`.

`dry-run` should:

- Hash resource.
- Reject unknown version unless `--developer` is present.
- Build a patch plan.
- Print files and edit counts.
- Never write target files.

- [ ] **Step 4: Run CLI checks**

Run:

```bash
cargo run -p omp-cli -- validate-data --manifest translations/maplelegends/manifest.toml --translations translations/maplelegends/zh-CN/item.jsonl
cargo run -p omp-cli -- dry-run --game-dir fixtures/synthetic-client --manifest translations/maplelegends/manifest.toml --translations translations/maplelegends/zh-CN/item.jsonl
```

Expected:

```text
validation ok
dry run ok
```

- [ ] **Step 5: Commit**

```bash
git add cli/src/main.rs cli/src/commands.rs
git commit -m "feat: validate data with cli"
```

## Chunk 6: Tauri GUI Shell

### Task 13: Scaffold GUI app

**Files:**
- Create: `tauri-app/package.json`
- Create: `tauri-app/src/App.tsx`
- Create: `tauri-app/src/main.tsx`
- Create: `tauri-app/src/api.ts`
- Create: `tauri-app/src/styles.css`
- Create: `tauri-app/src-tauri/Cargo.toml`
- Create: `tauri-app/src-tauri/tauri.conf.json`
- Create: `tauri-app/src-tauri/src/main.rs`
- Create: `tauri-app/src-tauri/src/commands.rs`
- Modify: `Cargo.toml`

- [ ] **Step 1: Add Tauri workspace member**

Add `tauri-app/src-tauri` to workspace members.

- [ ] **Step 2: Create GUI layout**

Build the screens:

- Install.
- Restore.
- Translation Data.
- Logs.
- Settings.
- About.

The install page initially uses mocked status:

```ts
type ClientStatus =
  | { kind: "notSelected" }
  | { kind: "supported"; version: string; installed: boolean }
  | { kind: "unsupported"; reason: string };
```

- [ ] **Step 3: Add Tauri commands**

Expose commands:

- `select_game_dir_status(path: string)`
- `install_localization(path: string, mode: "zh-CN" | "zh-CN-bilingual")`
- `restore_original(path: string)`
- `check_translation_data()`

Initial commands may return synthetic fixture results, but must call into `patch-core` where file operations are involved.

- [ ] **Step 4: Run GUI checks**

Run:

```bash
cd tauri-app
npm install
npm run build
cargo test -p open-maple-patch-tauri
```

Expected: frontend builds and Rust Tauri command tests pass.

- [ ] **Step 5: Commit**

```bash
git add Cargo.toml tauri-app
git commit -m "feat: add tauri assistant shell"
```

### Task 14: Wire GUI to synthetic install and restore

**Files:**
- Modify: `tauri-app/src/App.tsx`
- Modify: `tauri-app/src/api.ts`
- Modify: `tauri-app/src-tauri/src/commands.rs`

- [ ] **Step 1: Add GUI state tests**

Use Vitest for UI state helpers:

- Unsupported status disables install.
- Supported status enables install.
- Install success shows backup location.
- Restore success updates status.

- [ ] **Step 2: Run frontend tests to verify failure**

Run: `cd tauri-app && npm test -- --run`

Expected: tests fail until helpers are implemented.

- [ ] **Step 3: Implement state helpers and command calls**

The GUI must:

- Disable install before validation.
- Show unknown version message.
- Show changed file list before install.
- Show logs from command errors.

- [ ] **Step 4: Run GUI verification**

Run:

```bash
cd tauri-app
npm test -- --run
npm run build
```

Expected: tests and build pass.

- [ ] **Step 5: Commit**

```bash
git add tauri-app/src tauri-app/src-tauri/src
git commit -m "feat: connect gui to patch core"
```

## Chunk 7: Translation Data Update Flow

### Task 15: Add update manifest verification

**Files:**
- Create: `patch-core/src/report.rs`
- Create: `patch-core/src/update.rs`
- Modify: `patch-core/src/lib.rs`
- Test: `patch-core/tests/update_tests.rs`

- [ ] **Step 1: Write failing data update tests**

Test:

- A downloaded manifest with expected checksum is accepted.
- A mismatched checksum is rejected.
- Update records never include client resource paths as downloadable assets.

- [ ] **Step 2: Run test to verify failure**

Run: `cargo test -p patch-core --test update_tests`

Expected: compile fails because update module does not exist.

- [ ] **Step 3: Implement update metadata validation**

Use metadata shaped like:

```json
{
  "version": "2026.06.21",
  "manifest_url": "https://example.invalid/manifest.toml",
  "manifest_sha256": "...",
  "data_files": [
    {"path": "zh-CN/item.jsonl", "sha256": "..."}
  ]
}
```

Rules:

- Reject `.wz`, `.img`, `.exe`.
- Verify hashes before replacing local translation data.
- Write into app data cache, not game directory.

- [ ] **Step 4: Run tests**

Run: `cargo test -p patch-core --test update_tests`

Expected: update tests pass.

- [ ] **Step 5: Commit**

```bash
git add patch-core/src/lib.rs patch-core/src/report.rs patch-core/src/update.rs patch-core/tests/update_tests.rs
git commit -m "feat: validate translation data updates"
```

### Task 16: Expose data update in GUI

**Files:**
- Modify: `tauri-app/src/App.tsx`
- Modify: `tauri-app/src-tauri/src/commands.rs`

- [ ] **Step 1: Add mocked update UI test**

Test states:

- No update checked.
- Update available.
- Data current.
- Update failed with reason.

- [ ] **Step 2: Implement update page**

The page must say clearly:

- It updates translation data only.
- It does not update program binaries.
- It never downloads client files.

- [ ] **Step 3: Run GUI checks**

Run:

```bash
cd tauri-app
npm test -- --run
npm run build
```

Expected: tests and build pass.

- [ ] **Step 4: Commit**

```bash
git add tauri-app/src tauri-app/src-tauri/src
git commit -m "feat: add translation data update UI"
```

## Chunk 8: Documentation, Release Checks, And Review

### Task 17: Add contributor documentation

**Files:**
- Create: `docs/CONTRIBUTING.md`
- Create: `docs/TRANSLATION_DATA.md`
- Modify: `README.md`

- [ ] **Step 1: Document contribution rules**

Include:

- Do not submit MapleLegends client files.
- Do not submit paid localization data.
- Do not submit gameplay-affecting edits.
- Translation entries need stable keys and review status.
- Unknown version adaptation starts with CLI dry-run reports.

- [ ] **Step 2: Document local verification**

Commands:

```bash
cargo test --workspace
cargo run -p omp-cli -- validate-data --manifest translations/maplelegends/manifest.toml --translations translations/maplelegends/zh-CN/item.jsonl
cd tauri-app && npm test -- --run && npm run build
```

- [ ] **Step 3: Commit**

```bash
git add README.md docs/CONTRIBUTING.md docs/TRANSLATION_DATA.md
git commit -m "docs: add contribution guide"
```

### Task 18: Add CI

**Files:**
- Create: `.github/workflows/ci.yml`

- [ ] **Step 1: Add CI workflow**

CI jobs:

- Rust format check.
- Rust tests.
- CLI validation against synthetic fixtures.
- Frontend install, tests, and build.

- [ ] **Step 2: Run local equivalent**

Run:

```bash
cargo fmt --check
cargo test --workspace
cargo run -p omp-cli -- validate-data --manifest translations/maplelegends/manifest.toml --translations translations/maplelegends/zh-CN/item.jsonl
cd tauri-app && npm test -- --run && npm run build
```

Expected: all pass.

- [ ] **Step 3: Commit**

```bash
git add .github/workflows/ci.yml
git commit -m "ci: verify workspace"
```

### Task 19: Run final review

**Files:**
- Review all changed files.

- [ ] **Step 1: Run full verification**

Run:

```bash
cargo fmt --check
cargo test --workspace
cargo run -p omp-cli -- dry-run --game-dir fixtures/synthetic-client --manifest translations/maplelegends/manifest.toml --translations translations/maplelegends/zh-CN/item.jsonl
cd tauri-app && npm test -- --run && npm run build
```

Expected: all pass.

- [ ] **Step 2: Check repository for forbidden file types**

Run:

```bash
find . -type f | rg -n "\\.(wz|img|exe)$|MapleLegends\\.exe$"
```

Expected: no matches except documented placeholder files that do not use `.exe`.

- [ ] **Step 3: Check public claims**

Run:

```bash
rg -n "officially allowed|risk-free|不会封号|零风险|guaranteed|undetectable|bypass" README.md docs
```

Expected: no unsafe claims.

- [ ] **Step 4: Run `/review`**

Perform an independent review focusing on:

- File mutation safety.
- Restore correctness.
- Unknown version rejection.
- Forbidden scope.
- README and contributor boundary language.
- Test coverage for failure paths.

- [ ] **Step 5: Commit review fixes**

If fixes are needed:

```bash
git add <changed-files>
git commit -m "chore: address review findings"
```

## Real Client Adaptation Gate

Only after the synthetic vertical slice is complete:

- Use a user-provided local MapleLegends client directory.
- Record hashes in a private working note first.
- Add manifest hashes only when resource behavior is understood.
- Do not commit client resources.
- Do not publish instructions for bypassing protected resource access.
- Keep real adapter work behind `ResourceCodec`.

The first public release may ship with synthetic tests and an adapter scaffold if real resource support still needs validation.
