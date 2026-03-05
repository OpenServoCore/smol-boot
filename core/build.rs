fn main() {
    if std::env::var_os("RTT_LOG").is_some() {
        println!("cargo:rustc-cfg=rtt_log");
    }
    println!("cargo:rerun-if-env-changed=RTT_LOG");
    println!("cargo::rustc-check-cfg=cfg(rtt_log)");
}
