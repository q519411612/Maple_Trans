use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use patch_core::hash::sha256_file_hex;
use patch_core::length::{validate_text_length, TextTarget};
use patch_core::manifest::Manifest;
use patch_core::plan::{build_patch_plan, LanguageMode};
use patch_core::translation::parse_jsonl;

#[derive(Debug, Parser)]
#[command(name = "omp-cli")]
#[command(about = "Open Maple Patch maintainer tools")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    ValidateData {
        #[arg(long)]
        manifest: PathBuf,
        #[arg(long)]
        translations: PathBuf,
    },
    DryRun {
        #[arg(long)]
        game_dir: PathBuf,
        #[arg(long)]
        manifest: PathBuf,
        #[arg(long)]
        translations: PathBuf,
        #[arg(long)]
        developer: bool,
    },
}

pub fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Command::ValidateData {
            manifest,
            translations,
        } => validate_data(manifest, translations),
        Command::DryRun {
            game_dir,
            manifest,
            translations,
            developer,
        } => dry_run(game_dir, manifest, translations, developer),
    }
}

fn read_manifest(path: PathBuf) -> Result<Manifest> {
    let input = fs::read_to_string(&path)
        .with_context(|| format!("cannot read manifest: {}", path.display()))?;
    toml::from_str(&input).with_context(|| format!("cannot parse manifest: {}", path.display()))
}

fn read_translations(path: PathBuf) -> Result<Vec<patch_core::translation::TranslationEntry>> {
    let input = fs::read_to_string(&path)
        .with_context(|| format!("cannot read translations: {}", path.display()))?;
    parse_jsonl(&input).map_err(|error| anyhow!(error))
}

fn validate_data(manifest_path: PathBuf, translations_path: PathBuf) -> Result<()> {
    let manifest = read_manifest(manifest_path)?;
    let entries = read_translations(translations_path)?;
    let resource = manifest
        .resources
        .first()
        .ok_or_else(|| anyhow!("manifest has no resources"))?;

    for entry in &entries {
        validate_text_length(
            TextTarget {
                key: &entry.key,
                text: &entry.zh_cn,
            },
            resource.length_policy.clone(),
        )
        .map_err(|error| anyhow!(error))?;
        validate_text_length(
            TextTarget {
                key: &entry.key,
                text: &entry.bilingual,
            },
            resource.length_policy.clone(),
        )
        .map_err(|error| anyhow!(error))?;
    }

    println!("validation ok");
    Ok(())
}

fn dry_run(
    game_dir: PathBuf,
    manifest_path: PathBuf,
    translations_path: PathBuf,
    developer: bool,
) -> Result<()> {
    let manifest = read_manifest(manifest_path)?;
    let entries = read_translations(translations_path)?;
    let resource = manifest
        .resources
        .first()
        .ok_or_else(|| anyhow!("manifest has no resources"))?;
    let resource_path = game_dir.join(&resource.source);
    let actual_hash = sha256_file_hex(&resource_path).map_err(|error| anyhow!(error))?;

    if actual_hash != resource.hash && !developer {
        return Err(anyhow!("unsupported client version: {}", actual_hash));
    }

    let plan = build_patch_plan(resource, &entries, LanguageMode::SimplifiedChinese)
        .map_err(|error| anyhow!(error))?;
    println!("dry run ok: {} edits for {}", plan.edits.len(), plan.source);
    Ok(())
}
