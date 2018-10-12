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
plumber-rs = {git = "https://github.com/38/plumber-rs.git"}
```

**Step 3** Create the servlet object.

In Rust, a servlet is actually a trait. For the sync servlet, we can implemnet the trait `plumber_rs::servlet::sync_servlet`.

```rust
use plumber_rs::servlet::{SyncServlet, Unimplemented, ServletMode, ServletFuncResult, Bootstrap, success, fail};
use plumber_rs::protocol::{ProtocolModel, Untyped};

struct HelloServlet {}

impl SyncServlet for HelloServlet {
    no_protocol!();
    fn init(&mut self, _args:&[&str], _model:&mut Self::ProtocolType) -> ServletFuncResult 
    {
        plumber_log!(N "Hello World");
        return success();
    }
    fn exec(&mut self, _ti:TypeInstanceObject) -> ServletFuncResult  { success(); }
    fn cleanup(&mut self) -> ServletFuncResult { success(); }
}

```

**Step 4** Create the bootstrap type and export the entry point

```rust
struct BootstrapType{}

impl Bootstrap for BootstrapType {
    type SyncServletType = HelloServlet;
    type AsyncServletType = Unimplemented;
    fn get(_args:&[&str]) -> BootstrapResult<Self>
    {
        return Self::make_sync(HelloServlet{});
    }
}

export_bootstrap!(BootstrapType);
```

**Step 5** Build the servlet

To build the servlet successfully you should have Plumber installed. See the plumber install instruction at [here](https://plumberserver.com/index.html#documentation.compile).

If you have Plumber installed under the `/`, `/usr/`, `/usr/local` or your home directory, you should be able to compile the servlet successfully.
If you have some other install path, please specify the environment root with `ENVROOT` envrionment variable.

And to build the servlet, you can simple run `cargo build`

```bash
cargo build
```

# Full Servlet Code

```rust
#[macro_use]
extern crate plumber_rs;
extern crate libc;

use plumber_rs::servlet::{SyncServlet, Unimplemented, ServletFuncResult, Bootstrap, BootstrapResult, sucess, fail};
use plumber_rs::protocol::{ProtocolModel, Untyped};

struct HelloServlet {}

impl SyncServlet for HelloServlet {
    no_protocol!();
    fn init(&mut self, _args:&[&str], _model:&mut Self::ProtocolType) -> ServletFuncResult 
    {
        plumber_log!(N "Hello World");
        return sucess();
    }
    fn exec(&mut self, _model:Self::DataModelType) -> ServletFuncResult  { sucess() }
    fn cleanup(&mut self) -> ServletFuncResult { sucess() }
}

struct BootstrapType{}

impl Bootstrap for BootstrapType {
    type SyncServletType = HelloServlet;
    type AsyncServletType = Unimplemented;
    fn get(_args:&[&str]) -> BootstrapResult
    {
        return Self::make_sync(HelloServlet{});
    }
}

export_bootstrap!(BootstrapType);
```
