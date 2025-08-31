use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};

pub struct Config {
    pub reason: Option<String>,
    pub colored: bool,
    pub show_timestamp: bool,
    pub fps: u64,
}
impl Config {
    pub fn new() -> Config {
        let app = create_app();
        let matches = app.get_matches();
        Config {
            reason: matches.value_of("reason").map(|s| s.to_string()),
            colored: !matches.is_present("without_color"),
            show_timestamp: !matches.is_present("without_timestamp"),
            fps: match matches.value_of("speed") {
                Some("slow") => 150,
                Some("normal") => 100,
                Some("fast") => 75,
                _ => panic!("Invalid speed"),
            },
        }
    }

    pub fn is_exist_footer(&self) -> bool {
        self.show_timestamp || self.reason.is_some()
    }
}

fn create_app<'a, 'b>() -> App<'a, 'b> {
    App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(Arg::with_name("reason").index(1))
        .arg(
            Arg::with_name("without_color")
                .long("without-color")
                .short("C")
                .help("Disable color output"),
        )
        .arg(
            Arg::with_name("without_timestamp")
                .long("without-timestamp")
                .short("T")
                .help("Disable timestamp output"),
        )
        .arg(
            Arg::with_name("speed")
                .long("speed")
                .short("s")
                .help("Set the speed of the animation")
                .takes_value(true)
                .possible_value("fast")
                .possible_value("normal")
                .possible_value("slow")
                .default_value("normal")
                .hide_default_value(true),
        )
}
