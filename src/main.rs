use bevy::prelude::*;

mod sim_plugin;

use sim_plugin::SimPlugin;

fn main() {
    println!("Hello, world!");
    App::new().add_plugins(SimPlugin).run();
}
