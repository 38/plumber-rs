# plumber-rs

This is the Rust library that provides the connection between [Plumber Dataflow Framework](https://github.com/38/plumber)
and the Rust Programming language.

With this library, the servlet can be written in Rust programming language. 
Unlike C and C++, the Rust servlet needs a runtime environment, which can be setup with the Rust Servlet Loader `language/rust`.

## How to build a Rust servlet

With Cargo you should be able to create a servlet with Rust quite easily.

**Step 1** you should create a library crate with cargo

```
cargo init hello-rust --lib
```

**Step 2** we should change the `Cargo.toml`. 

We need to make sure the cargo can produce a dynamic library that can be loaded by Plumber Rust Servlet Loader.
Also, we need to add some depdendencies to the crate.

```
[lib]
name = "hello-rust"
crate-type= ["cdylib"]

[dependencies]
libc="0.2.0"
plumber-rs = {git = "https://github.com/38/plumber-rs.git"}
```

**Step 3** Create the servlet object.

In Rust, a servlet is actually a trait. For the sync servlet, we can implemnet the trait `plumber_rs::servlet::sync_servlet`.

```rust
use plumber_rs::servlet::{SyncServlet, Unimplemented, ServletMode, ServletFuncResult, Bootstrap};

struct HelloServlet {}

impl SyncServlet for HelloServlet {
    fn init(&mut self, _args:&[&str]) -> ServletFuncResult 
    {
        plumber_log!(N "Hello World");
        return Ok(());
    }
    fn exec(&mut self) -> ServletFuncResult  { Ok(()) }
    fn cleanup(&mut self) -> ServletFuncResult { Ok(()) }
}

```

**Step 4** Create the bootstrap type and export the entry point

```rust
struct BootstrapType{}

impl Bootstrap for BootstrapType {
    type SyncServletType = HelloServlet;
    type AsyncServletType = Unimplemented;
    fn get(_args:&[&str]) -> Result<ServletMode<Unimplemented, HelloServlet>, ()>
    {
        return Ok(ServletMode::SyncMode(HelloServlet{}));
    }
}

export_bootstrap!(BootstrapType);
```

**Step 4** Build the servlet

To build the servlet you should export two environment variable `PSTD_LIB_PATH` which is the path for libpstd.

And to build the servlet, you can simple run `cargo build`

```bash
export PSTD_LIB_PATH="/usr/lib"
cargo build
```

# Full Servlet Code

```rust
#[macro_use]
extern crate plumber_rs;
extern crate libc;

use plumber_rs::servlet::{SyncServlet, Unimplemented, ServletMode, ServletFuncResult, Bootstrap};

struct HelloServlet {}

impl SyncServlet for HelloServlet {
    fn init(&mut self, _args:&[&str]) -> ServletFuncResult 
    {
        plumber_log!(N "Hello World");
        return Ok(());
    }
    fn exec(&mut self) -> ServletFuncResult  { Ok(()) }
    fn cleanup(&mut self) -> ServletFuncResult { Ok(()) }
}

struct BootstrapType{}

impl Bootstrap for BootstrapType {
    type SyncServletType = HelloServlet;
    type AsyncServletType = Unimplemented;
    fn get(_args:&[&str]) -> Result<ServletMode<Unimplemented, HelloServlet>, ()>
    {
        return Ok(ServletMode::SyncMode(HelloServlet{}));
    }
}

export_bootstrap!(BootstrapType);
```
