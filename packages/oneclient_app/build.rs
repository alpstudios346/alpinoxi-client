fn main() {
    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("icons/icon.ico");

        let version = env!("CARGO_PKG_VERSION");
        res.set("CompanyName", "Alpinoxi");
        res.set("ProductName", "AlpinoxiClient");
        res.set("FileDescription", "AlpinoxiClient");
        res.set(
            "LegalCopyright",
            "© 2026 Alpinoxi contributors; see ATTRIBUTION.md for upstream credits.",
        );
        res.set("OriginalFilename", "alpinoxi_client.exe");
        res.set("InternalName", "alpinoxi_client");
        res.set("FileVersion", version);
        res.set("ProductVersion", version);

        res.compile()
            .expect("failed to embed Windows icon resource");
    }
}
