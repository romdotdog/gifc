use std::{io::stdout, process::exit};

use atty::Stream;
use clap::{App, Arg};
use std::fs::File;

mod gif;

fn main() {
    let matches = App::new("gifc")
        .version("1.0")
        .author("romdotdog")
        .about("caption gifs like a pro 😎")
        .arg(
            Arg::with_name("INPUT")
                .help("input gif url or fs path")
                .required(true),
        )
        .arg(
            Arg::with_name("caption")
                .help("sets the caption")
                .required(true),
        )
        .arg(
            Arg::with_name("force")
                .short("f")
                .long("force")
                .help("bypasses the interactive terminal check"),
        )
        .get_matches();

    if !matches.is_present("force") && atty::is(Stream::Stdout) {
        eprintln!("no pipe detected. did you mean to use `> out.gif`?");
        eprintln!("use the -f or --force command to continue");
        exit(1);
    }

    gif::caption(
        File::open(matches.value_of("INPUT").unwrap()).unwrap(),
        stdout(),
        matches.value_of("caption").unwrap().to_owned(),
    )
}
