pub mod modules;

use modules::config;
fn main() {
   let paths = config::deserialise_config();
   dbg!(&paths);
}