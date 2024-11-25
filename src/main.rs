pub mod modules;

use modules::config;
use crate::modules::watcher::watcher;

fn main() {
   let paths = config::deserialize_config();
   // dbg!(&paths);
   match paths {
      Ok(config) => {
         let watch_paths = config::extract_paths(&config);
         println!("{:?}", watch_paths);
         watcher(watch_paths);
      }
      Err(err) => {
         panic!("{}", err);
      }
   }
}

