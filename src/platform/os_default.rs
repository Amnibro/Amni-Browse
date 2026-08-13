use std::path::PathBuf;
use std::process::Command;
pub fn exe_path() -> PathBuf {
    std::env::current_exe().unwrap_or_else(|_| PathBuf::from("amni-browse.exe"))
}
pub fn open_command_line(exe: &str) -> String {
    format!("\"{}\" \"%1\"", exe)
}
#[cfg(target_os = "windows")]
pub fn register_browser() -> Result<String, String> { register_browser_exe(&exe_path()) }
#[cfg(target_os = "windows")]
pub fn register_browser_exe(exe: &std::path::Path) -> Result<String, String> {
    let exe_s = exe.to_string_lossy().to_string();
    let cmd = open_command_line(&exe_s);
    let icon = format!("{},0", exe_s);
    let pairs = [
        (r"HKCU\Software\Classes\AmniBrowseHTML", "", "Amni Browse HTML"),
        (r"HKCU\Software\Classes\AmniBrowseHTML\DefaultIcon", "", &icon),
        (r"HKCU\Software\Classes\AmniBrowseHTML\shell\open\command", "", &cmd),
        (r"HKCU\Software\Clients\StartMenuInternet\AmniBrowse", "", "Amni Browse"),
        (r"HKCU\Software\Clients\StartMenuInternet\AmniBrowse\DefaultIcon", "", &icon),
        (r"HKCU\Software\Clients\StartMenuInternet\AmniBrowse\shell\open\command", "", &format!("\"{}\"", exe_s)),
        (r"HKCU\Software\Clients\StartMenuInternet\AmniBrowse\Capabilities", "ApplicationName", "Amni Browse"),
        (r"HKCU\Software\Clients\StartMenuInternet\AmniBrowse\Capabilities", "ApplicationIcon", &icon),
        (r"HKCU\Software\Clients\StartMenuInternet\AmniBrowse\Capabilities", "ApplicationDescription", "Privacy-first Servo browser by Amni-Scient"),
        (r"HKCU\Software\Clients\StartMenuInternet\AmniBrowse\Capabilities\URLAssociations", "http", "AmniBrowseHTML"),
        (r"HKCU\Software\Clients\StartMenuInternet\AmniBrowse\Capabilities\URLAssociations", "https", "AmniBrowseHTML"),
        (r"HKCU\Software\Clients\StartMenuInternet\AmniBrowse\Capabilities\FileAssociations", ".html", "AmniBrowseHTML"),
        (r"HKCU\Software\Clients\StartMenuInternet\AmniBrowse\Capabilities\FileAssociations", ".htm", "AmniBrowseHTML"),
        (r"HKCU\Software\RegisteredApplications", "AmniBrowse", r"Software\Clients\StartMenuInternet\AmniBrowse\Capabilities"),
    ];
    for (key, value, data) in pairs {
        let mut c = Command::new("reg");
        c.arg("add").arg(key).arg("/f").arg("/d").arg(data);
        if !value.is_empty() { c.arg("/v").arg(value); }
        c.status().map_err(|e| e.to_string())?.success().then_some(()).ok_or_else(|| format!("reg add failed: {}", key))?;
    }
    let _ = Command::new("cmd").args(["/C", "start", "", "ms-settings:defaultapps"]).status();
    Ok("Registered Amni Browse. Pick it as the HTTP/HTTPS default in Windows Settings.".into())
}
#[cfg(not(target_os = "windows"))]
pub fn register_browser() -> Result<String, String> {
    Err("Set-as-default is wired for Windows. On this OS, set Amni Browse in system settings.".into())
}
pub fn open_path(path: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    { Command::new("cmd").args(["/C", "start", "", path]).spawn().map(|_| ()).map_err(|e| e.to_string()) }
    #[cfg(target_os = "macos")]
    { Command::new("open").arg(path).spawn().map(|_| ()).map_err(|e| e.to_string()) }
    #[cfg(all(unix, not(target_os = "macos")))]
    { Command::new("xdg-open").arg(path).spawn().map(|_| ()).map_err(|e| e.to_string()) }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn open_command_quotes_exe_and_arg() {
        assert_eq!(open_command_line(r"C:\Program Files\Amni\amni-browse.exe"), r#""C:\Program Files\Amni\amni-browse.exe" "%1""#);
    }
}
