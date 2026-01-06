fn main() {
    // Link against the Objective-C runtime on macOS
    println!("cargo:rustc-link-lib=objc");
    
    // Link against AppKit framework for GUI components
    println!("cargo:rustc-link-lib=framework=AppKit");
    
    // Link against Foundation framework
    println!("cargo:rustc-link-lib=framework=Foundation");
}
