use std::{env, fs, path::{Path, PathBuf}};
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=assets/chrome/toolbar.html");
    println!("cargo:rerun-if-changed=assets/amni-browse.ico");
    println!("cargo:rerun-if-env-changed=CARGO_PKG_VERSION");
    println!("cargo:rerun-if-env-changed=GSTREAMER_1_0_ROOT_MSVC_X86_64");
    #[cfg(target_os = "windows")]
    {
        embed_pe_version();
        copy_angle_dlls();
        copy_gstreamer_plugins();
        stage_chrome_assets();
    }
}
#[cfg(target_os = "windows")]
fn embed_pe_version() {
    let ver = env::var("CARGO_PKG_VERSION").unwrap_or_else(|_| "0.0.0".into());
    let mut it = ver.split('.').filter_map(|s| s.parse::<u16>().ok());
    let (maj, min, pat) = (it.next().unwrap_or(0), it.next().unwrap_or(0), it.next().unwrap_or(0));
    let manifest = env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".into());
    let icon = Path::new(&manifest).join("assets").join("amni-browse.ico");
    let icon_esc = icon.to_string_lossy().replace('\\', "\\\\");
    let fv = format!("{}.{}.{}.0", maj, min, pat);
    let body = format!(
        "1 ICON \"{icon}\"\n1 VERSIONINFO\nFILEVERSION {maj},{min},{pat},0\nPRODUCTVERSION {maj},{min},{pat},0\nFILEFLAGSMASK 0x3fL\nFILEFLAGS 0x0L\nFILEOS 0x40004L\nFILETYPE 0x1L\nFILESUBTYPE 0x0L\nBEGIN\n    BLOCK \"StringFileInfo\"\n    BEGIN\n        BLOCK \"040904b0\"\n        BEGIN\n            VALUE \"CompanyName\", \"Amni-Scient\"\n            VALUE \"FileDescription\", \"Amni Browse\"\n            VALUE \"FileVersion\", \"{fv}\"\n            VALUE \"InternalName\", \"amni-browse\"\n            VALUE \"LegalCopyright\", \"Amni-Scient\"\n            VALUE \"OriginalFilename\", \"amni-browse.exe\"\n            VALUE \"ProductName\", \"Amni Browse\"\n            VALUE \"ProductVersion\", \"{fv}\"\n        END\n    END\n    BLOCK \"VarFileInfo\"\n    BEGIN\n        VALUE \"Translation\", 0x409, 1200\n    END\nEND\n",
        icon = icon_esc, maj = maj, min = min, pat = pat, fv = fv
    );
    let out = PathBuf::from(env::var("OUT_DIR").unwrap_or_else(|_| ".".into())).join("amni_pe_version.rc");
    if fs::write(&out, body).is_ok() {
        let _ = embed_resource::compile(&out, embed_resource::NONE);
    }
}
#[cfg(target_os = "windows")]
fn profile_dir() -> Option<PathBuf> {
    let out = env::var("OUT_DIR").ok()?;
    PathBuf::from(out).ancestors().nth(3).map(|p| p.to_path_buf())
}
#[cfg(target_os = "windows")]
fn stage_chrome_assets() {
    let Some(profile_dir) = profile_dir() else { return };
    let src_dir = Path::new("assets/chrome");
    let dst_dir = profile_dir.join("assets").join("chrome");
    if fs::create_dir_all(&dst_dir).is_err() { println!("cargo:warning=could not create {}", dst_dir.display()); return; }
    let entries = match fs::read_dir(src_dir) { Ok(e) => e, Err(_) => { println!("cargo:warning=missing {}", src_dir.display()); return; } };
    let mut staged = 0usize;
    for entry in entries.flatten() {
        let src = entry.path();
        if !src.is_file() { continue; }
        let dst = dst_dir.join(entry.file_name());
        fs::copy(&src, &dst).is_ok().then(|| staged += 1);
    }
    println!("cargo:warning=staged {} chrome asset(s) to {}", staged, dst_dir.display());
}
#[cfg(target_os = "windows")]
fn copy_angle_dlls() {
    let Some(profile_dir) = profile_dir() else { return };
    let build_dir = profile_dir.join("build");
    if !build_dir.exists() { return; }
    let names = ["libEGL.dll", "libGLESv2.dll"];
    let entries = match fs::read_dir(&build_dir) { Ok(e) => e, Err(_) => return };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().to_string();
        if !name.starts_with("mozangle-") { continue; }
        let dll_dir = entry.path().join("out");
        for n in &names {
            let src = dll_dir.join(n);
            let dst = profile_dir.join(n);
            if src.exists() && fs::copy(&src, &dst).is_ok() { println!("cargo:warning=copied {} -> {}", n, dst.display()); }
        }
    }
}
#[cfg(target_os = "windows")]
fn copy_gstreamer_plugins() {
    let Some(profile_dir) = profile_dir() else { return };
    let Ok(gst_root) = env::var("GSTREAMER_1_0_ROOT_MSVC_X86_64") else { return };
    copy_gstreamer_runtime_dlls(&gst_root, &profile_dir);
    let plugin_dir = PathBuf::from(&gst_root).join("lib").join("gstreamer-1.0");
    if !plugin_dir.exists() { println!("cargo:warning=gstreamer plugin dir not found: {}", plugin_dir.display()); return; }
    let plugins = [
        "gstcoreelements", "gstnice", "gstapp", "gstaudioconvert", "gstaudioresample",
        "gstgio", "gstogg", "gstopengl", "gstopus", "gstplayback", "gsttheora",
        "gsttypefindfunctions", "gstvideoconvertscale", "gstvolume", "gstvorbis",
        "gstaudiofx", "gstaudioparsers", "gstautodetect", "gstdeinterlace",
        "gstid3demux", "gstinterleave", "gstisomp4", "gstmatroska", "gstrtp",
        "gstrtpmanager", "gstvideofilter", "gstvpx", "gstwavparse",
        "gstaudiobuffersplit", "gstdtls", "gstid3tag", "gstproxy",
        "gstvideoparsersbad", "gstwebrtc", "gstlibav", "gstwasapi",
    ];
    let mut copied = 0usize;
    for p in &plugins {
        let name = format!("{}.dll", p);
        let src = plugin_dir.join(&name);
        if !src.exists() { println!("cargo:warning=missing gstreamer plugin: {}", src.display()); continue; }
        let dst = profile_dir.join(&name);
        if fs::copy(&src, &dst).is_ok() { copied += 1; }
    }
    println!("cargo:warning=copied {} gstreamer plugins to {}", copied, profile_dir.display());
}
#[cfg(target_os = "windows")]
fn copy_gstreamer_runtime_dlls(gst_root: &str, profile_dir: &Path) {
    let bin_dir = PathBuf::from(gst_root).join("bin");
    let entries = match fs::read_dir(&bin_dir) { Ok(e) => e, Err(_) => { println!("cargo:warning=gstreamer bin dir not found: {}", bin_dir.display()); return; } };
    let mut copied = 0usize;
    for entry in entries.flatten() {
        let src = entry.path();
        if src.extension().and_then(|e| e.to_str()) != Some("dll") { continue; }
        let dst = profile_dir.join(entry.file_name());
        fs::copy(&src, &dst).is_ok().then(|| copied += 1);
    }
    println!("cargo:warning=copied {} gstreamer runtime dll(s) to {} (so the exe launches without run.bat's PATH shim)", copied, profile_dir.display());
}
