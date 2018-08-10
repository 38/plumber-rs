extern crate bindgen;

use std::env;
use std::path::PathBuf;

fn main() 
{
    let pservlet_prefix = env::var("PSERVLET_INSTALL_PREFIX").unwrap_or("".to_string());

    let include_dir = pservlet_prefix + "/include/pservlet";

    eprintln!("{}", include_dir);

    let bindings = bindgen::Builder::default()
        .header("include/plumber_api.h")
        .clang_args(["-I", include_dir.as_str()].iter())
        .generate()
        .expect("Unable to generate bindings");

    // Write the bindings to the $OUT_DIR/bindings.rs file.
    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_path.join("plumber_api_binding.rs"))
            .expect("Couldn't write bindings!");
}
