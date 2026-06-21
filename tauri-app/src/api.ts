import { invoke } from "@tauri-apps/api/core";

export type LanguageMode = "zh-CN" | "zh-CN-bilingual";

export type ClientStatus =
  | { kind: "notSelected" }
  | { kind: "supported"; version: string; installed: boolean; changedFiles: string[] }
  | { kind: "unsupported"; reason: string };

export type OperationResult = {
  ok: boolean;
  message: string;
  backupPath?: string;
};

export function canInstall(status: ClientStatus): boolean {
  return status.kind === "supported";
}

export async function inspectGameDir(path: string): Promise<ClientStatus> {
  return invoke<ClientStatus>("select_game_dir_status", { path });
}

export async function installLocalization(path: string, mode: LanguageMode): Promise<OperationResult> {
  return invoke<OperationResult>("install_localization", { path, mode });
}

export async function restoreOriginal(path: string): Promise<OperationResult> {
  return invoke<OperationResult>("restore_original", { path });
}

export async function checkTranslationData(): Promise<OperationResult> {
  return invoke<OperationResult>("check_translation_data");
}
