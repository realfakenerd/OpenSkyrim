fn main() {
    cc::Build::new()
        .cpp(true)
        .flag_if_supported("/std:c++14")
        .flag_if_supported("-std=c++14")
        .file("src/basis_ktx2_bridge.cpp")
        .compile("opensky_basis_ktx2_bridge");
    println!("cargo:rerun-if-changed=src/basis_ktx2_bridge.cpp");
}
