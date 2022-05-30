use std::io::Result;
use prost_build;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=proto");
    prost_build::compile_protos(&["proto/messages.proto"], &["proto/"])?;
    Ok(())
}
