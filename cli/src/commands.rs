use std::fs;
use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use clap::{Parser, Subcommand};
use patch_core::hash::sha256_file_hex;
use patch_core::length::{validate_text_length, TextTarget};
use patch_core::manifest::Manifest;
use patch_core::plan::{build_patch_plan, LanguageMode};
use patch_core::resource::export::{
    collect_export_entries, target_resources, validate_export_client_dir, write_export_jsonl,
};
use patch_core::resource::wz_img::read_img_text_nodes;
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
    ExportText {
        #[arg(long)]
        game_dir: PathBuf,
        #[arg(long)]
        out: PathBuf,
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
        Command::ExportText { game_dir, out } => export_text(game_dir, out),
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

fn export_text(game_dir: PathBuf, out_dir: PathBuf) -> Result<()> {
    validate_export_client_dir(&game_dir).map_err(|error| anyhow!(error))?;

    let mut total_entries = 0usize;
    for target in target_resources() {
        let resource_path = game_dir.join(target.relative_path);
        let hash = sha256_file_hex(&resource_path).map_err(|error| anyhow!(error))?;
        let nodes = read_img_text_nodes(&game_dir, *target).map_err(|error| anyhow!(error))?;
        let entries = collect_export_entries(nodes).map_err(|error| anyhow!(error))?;
        let file_name = format!("{}.jsonl", target.id);
        let output_path = write_export_jsonl(&out_dir, &file_name, &entries)
            .with_context(|| format!("cannot write export file: {}", file_name))?;

        total_entries += entries.len();
        println!(
            "exported {} entries from {} ({}) to {}",
            entries.len(),
            target.relative_path,
            hash,
            output_path.display()
        );
    }

    println!(
        "export ok: {} resources, {} entries",
        target_resources().len(),
        total_entries
    );
    Ok(())
}
