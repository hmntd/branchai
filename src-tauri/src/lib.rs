use tauri::Manager;

pub mod services;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let icon_bytes = include_bytes!("../icons/icon.png");
            if let Ok(icon) = tauri::image::Image::from_bytes(icon_bytes) {
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.set_icon(icon);
                }
            }
            Ok(())
        })
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            greet,
            services::git_service::pick_repository_folder,
            services::git_service::get_commits,
            services::git_service::get_branches,
            services::git_service::get_repo_status,
            services::git_service::get_file_diff,
            services::git_service::get_all_staged_diff,
            services::git_service::stage_file,
            services::git_service::unstage_file,
            services::git_service::stage_all,
            services::git_service::create_commit,
            services::git_service::get_graph_data,
            services::git_service::get_conflicts,
            services::git_service::resolve_conflict,
            services::git_service::checkout_branch,
            services::git_service::create_new_branch,
            services::git_service::undo_commit,
            services::git_service::redo_commit,
            services::git_service::stash_save,
            services::git_service::stash_pop,
            services::git_service::pull_changes,
            services::git_service::push_changes,
            services::git_service::discard_file_changes,
            services::git_service::stash_file,
            services::git_service::get_file_history,
            services::git_service::get_file_blame,
            services::git_service::open_in_external_diff,
            services::git_service::open_in_vscode,
            services::git_service::open_file_default,
            services::git_service::show_in_folder,
            services::git_service::create_patch_from_file,
            services::git_service::delete_file,
            services::git_service::get_commit_files,
            services::git_service::get_commit_file_diff,
            services::git_service::get_repo_remote_info,
            services::git_service::get_branch_diff_summary,
            services::git_service::merge_branch,
            services::git_service::get_repo_prs,
            services::git_service::save_repo_prs,
            services::git_service::get_repo_issues,
            services::git_service::save_repo_issues,
            services::git_service::apply_repo_account,
            services::git_service::test_account_connection,
            services::ai_service::generate_commit_message,
            services::ai_service::analyze_code_review,
            services::ai_service::assist_conflict_resolution,
            services::ai_service::test_ai_connection,
            services::config_service::get_config,
            services::config_service::save_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
