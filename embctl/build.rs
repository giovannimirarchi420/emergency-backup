use std::env;
use winresource::WindowsResource;

fn main() {
    if env::var_os("CARGO_CFG_WINDOWS").is_some() {
        match WindowsResource::new().set_icon("assets/cli.ico").compile() {
            Ok(_) => {}
            Err(err) => { eprintln!("Error for icon setting: {:?}", err ) }
        }
    }

}
