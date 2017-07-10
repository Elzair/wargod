extern crate dacite;
extern crate dacite_winit;
extern crate winit;
#[macro_use] extern crate glsl_to_spirv_macros;
#[macro_use] extern crate glsl_to_spirv_macros_impl;

use std::process;

pub mod renderer;

fn main() {
    match renderer::real_main() {
        Ok(_) => process::exit(0),
        Err(_) => process::exit(1),
    }
}
