#[macro_use]
extern crate err_derive;
#[macro_use]
extern crate itertools;

use model_data::ModelData;
use crate::assets::Assets;

mod assets;
mod game;
mod renderer;
mod shader;
mod model_data;
mod conversions;

fn main() {
    let assets = Assets::load().unwrap();
    game::run("Voids", &assets);
}
