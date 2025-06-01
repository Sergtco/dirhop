use std::{env, io, process::exit};

use crossterm::tty::IsTty;

use dirhop::{
    app::App,
    args::{Opts, usage},
};

fn main() -> io::Result<()> {
    if !io::stderr().is_tty() {
        eprintln!("This is not a terminal!");
        exit(1);
    }

    let opts = Opts::from_args(env::args()).unwrap_or_else(|err| {
        eprintln!("Argument error: {err}.");
        usage();
        exit(1)
    });

    let mut app = App::new(opts)?;
    let out = app.run();
    app.restore()?;

    if let Ok(out) = out {
        println!("{out}");
    }

    Ok(())
}
