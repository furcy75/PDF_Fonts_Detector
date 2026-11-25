fn main() {
    println!("cargo:rerun-if-changed=assets/app_icon.ico");
    println!("cargo:rerun-if-changed=assets/app_icon.png");

    #[cfg(windows)]
    {
        let mut res = winresource::WindowsResource::new();
        res.set_icon("assets/app_icon.ico");
        res.set("ProductName", "PDF Fonts Detector");
        res.set(
            "FileDescription",
            "Detect, download, extract and install PDF fonts",
        );
        res.compile().expect("Failed to compile Windows icon resource");
    }
}
