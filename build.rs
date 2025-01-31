// use std::env;
use std::io;
// use winres::WindowsResource;
// use winres::WindowsResource;

fn main() -> io::Result<()> {
    // Windows
    // this is causing problems so I'm removing it for now
    // if env::var_os("CARGO_CFG_WINDOWS").is_some() {
    //     let mut res = WindowsResource::new();
    //     res.set_icon("icon.ico");
    //     res.compile()?;
    // }

    // macOS - todo
    // unix  - todo

    Ok(())
}
