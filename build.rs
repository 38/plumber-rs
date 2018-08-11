use std::env;
fn main() 
{
    if let Ok(search_path) = env::var("PSTD_LIB_PATH")
    {
        println!("cargo:rustc-link-search={}", search_path);
    }
    println!("cargo:rustc-link-lib=pstd");
}
