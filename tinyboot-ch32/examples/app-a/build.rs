fn main() {
    println!("cargo:rustc-link-arg=-Tboot_request.x");
    if cfg!(feature = "defmt") {
        println!("cargo:rustc-link-arg=-Tdefmt.x");
    }
}
