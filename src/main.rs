pub mod modules;

use modules::config;
fn main() {
   let paths = config::deserialize_config();
   dbg!(&paths);
}