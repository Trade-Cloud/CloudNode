//! Compiles `ui/app.slint` (and everything it imports) into the Rust that `src/lib.rs` includes.

fn main() -> Result<(), slint_build::CompileError> {
    slint_build::compile("ui/app.slint")
}
