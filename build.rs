use std::io::{Result, Error, ErrorKind};
use std::collections::HashSet;

use prost_build;

fn main() -> Result<()> {
    compile_protos()
}

fn compile_protos() -> Result<()> {
    // println!("cargo:rerun-if-changed=proto");
    prost_build::compile_protos(&["proto/messages.proto"], &["proto/"])?;
    Ok(())
}
