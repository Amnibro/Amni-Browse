use serde::Serialize;
use log::info;
#[derive(Debug, Clone, Serialize)]
pub struct AmniApp {
    pub id: &'static str,
    pub name: &'static str,
    pub desc: &'static str,
    pub emoji: &'static str,
    pub launch: LaunchType,
    pub category: AppCategory,
}
#[derive(Debug, Clone, Serialize)]
pub enum LaunchType {
    Web(&'static str),
}
#[derive(Debug, Clone, Serialize)]
pub enum AppCategory { Web }
pub static AMNI_APPS: &[AmniApp] = &[
    AmniApp { id: "amni-scient", name: "Amni-Scient", desc: "Main website — all Amni products", emoji: "crown", launch: LaunchType::Web("https://amni-scient.com"), category: AppCategory::Web },
];
pub fn list_apps_json() -> String { "[]".into() }
pub fn site_url() -> &'static str { "https://amni-scient.com" }
pub fn launch_app(_id: &str) -> Result<String, String> {
    info!("Amni Apps routes to product site");
    Ok(site_url().to_string())
}
