use std::env;
use std::path::PathBuf;

fn main() {
    let _out_dir = env::var("OUT_DIR").unwrap();
    let workspace_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let good_c: PathBuf = PathBuf::from(&workspace_dir).join("good.c");
    
    println!("cargo:rerun-if-changed={}", good_c.display());
        
    cc::Build::new()
        .file(&good_c)
        .flag("-fno-stack-protector")
        .flag("-fno-delete-null-pointer-checks")
        .flag("-O0")
        .flag("-g")
        .flag("-DDEBUG")
        .compile("good");
    
    println!("cargo:rustc-link-lib=static=good");
}
