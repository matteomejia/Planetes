fn main() {
    println!("cargo:rerun-if-changed=assets/planetes.ico");
    println!("cargo:rerun-if-changed=assets/app.manifest");
    #[cfg(windows)]
    {
        let version = format!(
            "{}.{}.{}.0",
            env!("CARGO_PKG_VERSION_MAJOR"),
            env!("CARGO_PKG_VERSION_MINOR"),
            env!("CARGO_PKG_VERSION_PATCH")
        );
        let manifest = include_str!("assets/app.manifest").replace("{{VERSION}}", &version);
        winresource::WindowsResource::new()
            .set_icon_with_id("assets/planetes.ico", "1")
            .set_manifest(&manifest)
            .set("ProductName", "Planetes")
            .set("FileDescription", "Planetes game launcher")
            .set("OriginalFilename", "planetes.exe")
            .set("LegalCopyright", "Copyright (c) 2026 Francisco")
            .compile()
            .expect("compile Windows icon and version resources");
    }
}
