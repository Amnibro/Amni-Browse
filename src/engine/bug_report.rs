use crate::storage::config::{APP_NAME, APP_VERSION};
use urlencoding::encode;
pub fn github_new_issue_url(title: &str, user_body: &str, diag: &str) -> String {
    let t = if title.trim().is_empty() { "Amni Browse issue".to_string() } else { title.trim().chars().take(120).collect() };
    let mut body = String::new();
    body.push_str("## What happened\n\n");
    body.push_str(if user_body.trim().is_empty() { "_describe the problem_\n" } else { user_body.trim() });
    body.push_str("\n\n## Steps to reproduce\n\n1.\n2.\n3.\n\n## Expected\n\n\n## Actual\n\n\n");
    body.push_str("## Diagnostics (auto)\n\n```\n");
    body.push_str(diag);
    if !diag.ends_with('\n') { body.push('\n'); }
    body.push_str("```\n");
    format!(
        "https://github.com/Amnibro/Amni-Browse/issues/new?title={}&body={}",
        encode(&t),
        encode(&body)
    )
}
pub fn collect_diag(page_url: &str, extra: &str) -> String {
    let mut d = String::new();
    d.push_str(&format!("app: {} {}\n", APP_NAME, APP_VERSION));
    d.push_str(&format!("os: {} {}\n", std::env::consts::OS, std::env::consts::ARCH));
    d.push_str(&format!("page: {}\n", page_url.trim()));
    d.push_str(&format!("time_utc: {}\n", chrono::Utc::now().to_rfc3339()));
    if !extra.trim().is_empty() {
        d.push_str("extra:\n");
        d.push_str(extra.trim());
        d.push('\n');
    }
    d
}
