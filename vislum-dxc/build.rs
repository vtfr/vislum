fn main() {
    println!("cargo:rerun-if-changed=cpp/shim.cpp");
    println!("cargo:rerun-if-changed=cpp/");
    println!("cargo:rerun-if-changed=build.rs");

    cc::Build::new()
        .cpp(true)
        .flag_if_supported("-std=c++11")
        .file("cpp/shim.cpp")
        .compile("vislum-dxc-shim");
}
