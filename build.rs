fn main() {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=exhale-src");

    if cfg!(feature = "bundled") {
        let mut cfg = cmake::Config::new("exhale-src");
        cfg.build_target("exhaleLib");
        if cfg!(feature = "low-complexity") {
            cfg.cxxflag("-DEC_TRELLIS_OPT_CODING=0");
        }
        let install_dir = cfg.build();

        println!("cargo::rustc-link-lib=static=exhale");
        if cfg!(target_env = "msvc") {
            println!("cargo::rustc-link-lib=msvcrt");
            if cfg!(debug_assertions) {
                println!(
                    "cargo::rustc-link-search={}/build/src/lib/Debug",
                    install_dir.display()
                );
            } else {
                println!(
                    "cargo::rustc-link-search={}/build/src/lib/Release",
                    install_dir.display()
                );
            }
        } else {
            println!("cargo::rustc-link-lib=stdc++");
            println!(
                "cargo::rustc-link-search={}/build/src/lib",
                install_dir.display()
            );
        }
    } else {
        #[cfg(feature = "low-complexity")]
        compile_error!("the `low-complexity` feature is only available when using bundled library");

        println!("cargo::rustc-link-lib=dylib=exhale");
    }
}
