mod commands;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::select_game_dir_status,
            commands::install_localization,
            commands::restore_original,
            commands::check_translation_data
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Open Maple Patch");
}
