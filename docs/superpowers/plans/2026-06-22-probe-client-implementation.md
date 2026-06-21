# Probe Client Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a read-only `probe-client` CLI command that diagnoses MapleLegendsHD resource compatibility without extracting text dumps or modifying client files.

**Architecture:** Add probe data structures and file-format checks to `patch-core`, then expose a maintainer CLI command in `omp-cli`. The probe reports hashes, file headers, `list.wz` readability, and standalone `.img` parser attempts with known IV/version modes.

**Tech Stack:** Rust 2021, `serde`, `serde_json`, `wz_reader`, existing `patch-core` hashing helpers, `clap`.

---

## File Structure

- Create `patch-core/src/resource/probe.rs` for report models and deterministic pure helpers.
- Modify `patch-core/src/resource/mod.rs` to expose the probe module.
- Create `patch-core/tests/probe_tests.rs` for report serialization, magic/header formatting, and missing-file behavior.
- Modify `cli/src/commands.rs` to add `probe-client`.

## Chunk 1: Probe Model

### Task 1: Add report structures and pure helpers

**Files:**
- Create: `patch-core/src/resource/probe.rs`
- Modify: `patch-core/src/resource/mod.rs`
- Test: `patch-core/tests/probe_tests.rs`

- [ ] **Step 1: Write failing tests**

Test that `format_magic` renders the first bytes as lowercase hex, and that `ProbeReport` serializes deterministic JSON with client root, list result, and target resource results.

- [ ] **Step 2: Run tests and verify they fail**

Run:

```bash
cargo test -p patch-core --test probe_tests --locked
```

Expected: FAIL because the probe module does not exist.

- [ ] **Step 3: Implement minimal model and helpers**

Add `ProbeReport`, `ListProbe`, `ResourceProbe`, `ProbeAttempt`, `ProbeStatus`, and `format_magic`.

- [ ] **Step 4: Run tests and verify they pass**

Run:

```bash
cargo test -p patch-core --test probe_tests --locked
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add patch-core/src/resource/probe.rs patch-core/src/resource/mod.rs patch-core/tests/probe_tests.rs
git commit -m "feat: add client probe model"
```

## Chunk 2: Read-Only Probe Logic

### Task 2: Probe selected client files

**Files:**
- Modify: `patch-core/src/resource/probe.rs`
- Test: `patch-core/tests/probe_tests.rs`

- [ ] **Step 1: Write failing tests**

Test that missing `list.wz` and missing target `.img` files are reported in the model instead of aborting the whole probe.

- [ ] **Step 2: Run tests and verify they fail**

Run:

```bash
cargo test -p patch-core --test probe_tests --locked
```

Expected: FAIL because the filesystem probe function does not exist.

- [ ] **Step 3: Implement probe logic**

Implement `probe_client(game_dir: &Path) -> PatchResult<ProbeReport>`.
It must read files only, record SHA-256, record the first 16 bytes as magic, parse `list.wz` with GMS/EMS/BMS through available parser attempts, and try target `.img` parsing through `wz_reader` known versions and IV modes.

- [ ] **Step 4: Run tests and verify they pass**

Run:

```bash
cargo test -p patch-core --test probe_tests --locked
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add patch-core/src/resource/probe.rs patch-core/tests/probe_tests.rs
git commit -m "feat: add read-only client probe"
```

## Chunk 3: CLI Command And Verification

### Task 3: Add `probe-client`

**Files:**
- Modify: `cli/src/commands.rs`

- [ ] **Step 1: Verify missing command fails**

Run:

```bash
cargo run --locked -p omp-cli -- probe-client --game-dir fixtures/synthetic-client
```

Expected: FAIL because the command does not exist.

- [ ] **Step 2: Implement command**

Add `ProbeClient { game_dir: PathBuf }` and print pretty JSON from `probe_client`.

- [ ] **Step 3: Run automated verification**

Run:

```bash
cargo fmt --check
cargo test --workspace --locked
cargo clippy --workspace --locked -- -D warnings
```

Expected: PASS.

- [ ] **Step 4: Run real-client probe read-only**

Run:

```bash
cargo run --locked -p omp-cli -- probe-client --game-dir /Volumes/开发相关/MapleLegendsHD
```

Expected: JSON report exits 0 and shows `list.wz` GMS readable while target `.img` attempts fail with parser errors.

- [ ] **Step 5: Commit**

```bash
git add cli/src/commands.rs
git commit -m "feat: add probe client command"
```

## Final Verification

Run:

```bash
cargo fmt --check
cargo test --workspace --locked
cargo clippy --workspace --locked -- -D warnings
find . -type f \( -name '*.wz' -o -name '*.img' -o -name '*.exe' \) -print
```

Expected: all Rust checks pass and no client resources are present in the repository.
