use tauri::Manager;

pub mod models;
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
            services::git::pick_repository_folder,
            services::git::get_commits,
            services::git::get_branches,
            services::git::get_repo_status,
            services::git::get_file_diff,
            services::git::get_all_staged_diff,
            services::git::stage_file,
            services::git::unstage_file,
            services::git::stage_all,
            services::git::create_commit,
            services::git::get_graph_data,
            services::git::get_conflicts,
            services::git::resolve_conflict,
            services::git::checkout_branch,
            services::git::create_new_branch,
            services::git::undo_commit,
            services::git::redo_commit,
            services::git::stash_save,
            services::git::stash_pop,
            services::git::pull_changes,
            services::git::push_changes,
            services::git::discard_file_changes,
            services::git::stash_file,
            services::git::get_file_history,
            services::git::get_file_blame,
            services::git::open_in_external_diff,
            services::git::open_in_vscode,
            services::git::open_file_default,
            services::git::show_in_folder,
            services::git::create_patch_from_file,
            services::git::delete_file,
            services::git::get_commit_files,
            services::git::get_commit_file_diff,
            services::git::get_repo_remote_info,
            services::git::get_branch_diff_summary,
            services::git::merge_branch,
            services::git::get_repo_prs,
            services::git::save_repo_prs,
            services::git::get_repo_issues,
            services::git::save_repo_issues,
            services::git::apply_repo_account,
            services::git::test_account_connection,
            services::git::get_github_workflows,
            services::git::get_workflow_runs,
            services::git::get_workflow_run_jobs,
            services::git::get_job_logs,
            services::git::trigger_workflow_dispatch,
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
