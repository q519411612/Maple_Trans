# Open Maple Patch

Open Maple Patch is an unofficial text localization patcher for MapleLegends.

This project is not affiliated with MapleLegends or Nexon.

This project does not distribute MapleLegends client files. This project does not modify gameplay values, map collision, character state, packets, memory, or automation behavior. Use at your own risk.

The repository contains code, translation data, manifests, and synthetic test fixtures only. Do not upload original or modified MapleLegends resource files.

## Contributing

Read [`docs/CONTRIBUTING.md`](docs/CONTRIBUTING.md) before submitting code or translation data.

## Local Development

```bash
cargo test --workspace
cargo run -p omp-cli -- validate-data --manifest translations/maplelegends/manifest.toml --translations translations/maplelegends/zh-CN/item.jsonl
cargo run -p omp-cli -- dry-run --game-dir fixtures/synthetic-client --manifest translations/maplelegends/manifest.toml --translations translations/maplelegends/zh-CN/item.jsonl
```
