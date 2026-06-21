# Translation Data

Translation data lives under `translations/maplelegends`.

The repository uses:

- `manifest.toml` for supported versions, resource hashes, resource kinds, and length policies.
- `*.jsonl` files for translation entries.

Example entry:

```json
{"key":"item.2000000.name","source":"Red Potion","zh_CN":"红色药水","bilingual":"红药/Red","status":"reviewed"}
```

Rules:

- `key` must be stable and derived from resource structure.
- `source` is for review and validation.
- `zh_CN` is simplified Chinese.
- `bilingual` preserves the English lookup term.
- `status` is `draft` or `reviewed`.
- Over-length entries fail validation.
- The tool never silently truncates text.
