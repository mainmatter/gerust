#![allow(missing_docs)]
use std::error::Error;

use vergen::Emitter;
use vergen_git2::Git2Builder;

fn main() -> Result<(), Box<dyn Error>> {
    Emitter::default()
        .add_instructions(&Git2Builder::all_git()?)?
        .emit()?;

    println!("cargo:rerun-if-changed=blueprint");
    Ok(())
}
