use std::{error::Error, path::PathBuf, process::exit, result};

type Result<T> = result::Result<T, Box<dyn Error>>;

#[derive(Debug, Default)]
pub struct Opts {
    pub program_name: String,
    pub base_path: PathBuf,
    pub show_hidden: bool,
}

impl Opts {
    pub fn from_args(mut args: impl Iterator<Item = String>) -> Result<Self> {
        let mut conf = Self::default();
        conf.program_name = args.next().ok_or("Couldn't get program name")?;

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "-h" => conf.show_hidden = true,
                "--help" => {
                    usage();
                    exit(0)
                }
                someflag if someflag.starts_with("-") => {
                    return Err(format!("wrong flag: {someflag}").into());
                }
                rest => conf.base_path = PathBuf::try_from(&rest)?,
            }
        }

        if conf.base_path.to_string_lossy().len() == 0 {
            conf.base_path = ".".into();
        }

        Ok(conf)
    }
}

pub fn usage() {
    println!("USAGE:");
    println!("dirhop [PATH] [FLAGS]");
    println!("");
    println!("-h show hidden files");
    println!("--help show this help message");
}
