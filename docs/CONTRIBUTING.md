# Contributing

Open Maple Patch accepts code, manifests, synthetic fixtures, and original translation data.

Do not submit:

- MapleLegends client files.
- Original or modified resource files.
- Paid localization data.
- Gameplay-affecting edits.
- Memory, packet, automation, or bypass behavior.

Translation entries must use stable resource-derived keys and include review status.

Unknown version adaptation starts with CLI dry-run reports. Do not commit client resources while adapting a version.

## Local Checks

```bash
cargo test --workspace
cargo run -p omp-cli -- validate-data --manifest translations/maplelegends/manifest.toml --translations translations/maplelegends/zh-CN/item.jsonl
cargo run -p omp-cli -- validate-data --manifest translations/maplelegends/manifest.toml --translations translations/maplelegends/zh-CN-bilingual/item.jsonl
cargo run -p omp-cli -- dry-run --game-dir fixtures/synthetic-client --manifest translations/maplelegends/manifest.toml --translations translations/maplelegends/zh-CN/item.jsonl
cd tauri-app && pnpm exec vitest run && pnpm build
```
