#[macro_use]
extern crate err_derive;

use mesh::Mesh;
use crate::assets::Assets;

mod assets;
mod game;
mod renderer;
mod shader;
mod mesh;

fn main() {
    let assets = Assets::load().unwrap();
    game::run("Voids", &assets);
}
