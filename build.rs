use std::io::{Result, Error, ErrorKind};
use std::collections::HashSet;

use prost_build;

#[derive(Debug)]
struct IgnoreMacros(HashSet<String>);

impl bindgen::callbacks::ParseCallbacks for IgnoreMacros {
    fn will_parse_macro(&self, name: &str) -> bindgen::callbacks::MacroParsingBehavior {
        if self.0.contains(name) {
            bindgen::callbacks::MacroParsingBehavior::Ignore
        } else {
            bindgen::callbacks::MacroParsingBehavior::Default
        }
    }
}

fn main() -> Result<()> {
    compile_protos()?;
    compile_libservo()?;
    Ok(())
}

fn compile_protos() -> Result<()> {
    println!("cargo:rerun-if-changed=proto");
    prost_build::compile_protos(&["proto/messages.proto"], &["proto/"])?;
    Ok(())
}

fn compile_libservo() -> Result<()> {
    let servo_dir = "servo/src";
    println!("cargo:rerun-if-changed={}", servo_dir);
    let files = ["sccb_bus.c", "bcm283x_board_driver.c", "PCA9685_servo_driver.c"]
        .iter()
        .map(|s| format!("{}/{}", servo_dir, s))
        .collect::<Vec<String>>();
    cc::Build::new()
        .files(files)
        .include(servo_dir)
        .try_compile("servo")
        .map_err(|e| Error::new(ErrorKind::Other, e))?;

    let ignored_macros = vec!["FP_NAN".into(), "FP_NORMAL".into(), "FP_INFINITE".into(), "FP_SUBNORMAL".into(), "FP_ZERO".into(), "IPPORT_RESERVED".into()].into_iter().collect();

    let bindings = bindgen::builder()
        .header(format!("{}/PCA9685_servo_driver.h", servo_dir))
        .parse_callbacks(Box::new(IgnoreMacros(ignored_macros)))
        .generate()
        .map_err(|e| Error::new(ErrorKind::Other, e))?;

    bindings.write_to_file(format!("{}/servo.rs", std::env::var("OUT_DIR").unwrap()))
}
