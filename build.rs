use winres::WindowsResource;

fn main() {
    // Set app_id as environment variable for compile-time access
    println!("cargo:rustc-env=APP_ID=dev.zidane.screenoff");

    if cfg!(target_os = "windows") {
        WindowsResource::new()
            .set("CompanyName", "M Zidane")
            .set("FileDescription", "ScreenOff")
            .set("LegalCopyright", "Copyright (C) 2025 M Zidane")
            .set("ProductName", "ScreenOff")
            .set("ProductVersion", "1.0.0")
            .set("FileVersion", "1.0.0")
            .set("InternalName", "dev.zidane.screenoff")
            .set("OriginalFilename", "screenoff.exe")
            .set_icon("icons/app_icon.ico")
            .set_icon_with_id("icons/screen_on.ico", "101")
            .set_icon_with_id("icons/screen_off.ico", "102")
            .compile()
            .unwrap();
    }
}
