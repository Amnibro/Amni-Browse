use std::{env, fs, path::{Path, PathBuf}};
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-env-changed=GSTREAMER_1_0_ROOT_MSVC_X86_64");
    #[cfg(target_os = "windows")]
    {
        let rc = Path::new("assets/windows_app.rc");
        if rc.exists() { embed_resource::compile(rc, embed_resource::NONE); }
        copy_angle_dlls();
        copy_gstreamer_plugins();
    }
}
#[cfg(target_os = "windows")]
fn profile_dir() -> Option<PathBuf> {
    let out = env::var("OUT_DIR").ok()?;
    PathBuf::from(out).ancestors().nth(3).map(|p| p.to_path_buf())
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
            if src.exists() {
                let dst = profile_dir.join(n);
                if fs::copy(&src, &dst).is_ok() { println!("cargo:warning=copied {} -> {}", n, dst.display()); }
            }
        }
    }
}
#[cfg(target_os = "windows")]
fn copy_gstreamer_plugins() {
    let Some(profile_dir) = profile_dir() else { return };
    let Ok(gst_root) = env::var("GSTREAMER_1_0_ROOT_MSVC_X86_64") else { return };
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
