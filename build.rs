fn main() {
    #[cfg(target_os = "windows")]
    {
        let rc = std::path::Path::new("assets/windows_app.rc");
        if rc.exists() {
            embed_resource::compile(rc, embed_resource::NONE);
        }
    }
}
