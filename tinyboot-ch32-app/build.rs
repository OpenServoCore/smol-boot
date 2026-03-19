fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();
    std::fs::write(
        format!("{out_dir}/tinyboot.x"),
        include_bytes!("tinyboot.x"),
    )
    .unwrap();
    println!("cargo:rustc-link-search={out_dir}");
    println!("cargo:rerun-if-changed=tinyboot.x");
}
