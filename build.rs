use std::env;
use std::path::Path;
fn search_for_pstd_lib() -> Option<String>
{
    let mut candicates = vec!["/".to_string(), "/usr".to_string(), "/opt".to_string(), "/usr/local/".to_string()];

    if let Ok(home) = env::var("HOME")
    {
        candicates.push(home);
    }

    if let Ok(envroot) = env::var("ENVROOT")
    {
        candicates.push(envroot);
    }

    candicates.reverse();

    for path in candicates 
    {
        let lib_path = path.clone() + "/lib/libpstd.so";

        eprintln!("Looking for path {}", lib_path);

        if Path::new(&lib_path).exists() 
        {
            return Some(path + "/lib/");
        }
    }

    return None;
}

fn main() 
{
    if let Ok(search_path) = env::var("PSTD_LIB_PATH")
    {
        println!("cargo:rustc-link-search={}", search_path);
    }
    else if let Some(lib_path) = search_for_pstd_lib() 
    {
        println!("cargo:rustc-link-search={}", lib_path);
    }
    else
    {
        eprintln!("Cannot find libpstd.so, plumber-rs crate could not be built."
        eprintln!("Hint: make sure you have plumber intalled or try to set PSTD_LIB_PATH environment variable");
    }
    println!("cargo:rustc-link-lib=pstd");
}
