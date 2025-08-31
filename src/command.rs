use clap::{builder::styling, Parser};

pub struct Config {
    pub reason: Option<String>,
    pub colored: bool,
    pub show_timestamp: bool,
    pub fps: u64,
}
impl Config {
    pub fn new() -> Config {
        let args = Args::parse();
        Config {
            reason: args.reason,
            colored: !args.without_color,
            show_timestamp: !args.without_timestamp,
            fps: match args.speed.as_str() {
                "slow" => 150,
                "normal" => 100,
                "fast" => 75,
                _ => panic!("Invalid speed"),
            },
        }
    }

    pub fn is_exist_footer(&self) -> bool {
        self.show_timestamp || self.reason.is_some()
    }
}

#[derive(Parser, Debug)]
#[command(author, version, about, styles = get_styles())]
pub struct Args {
    #[arg(help = "Reason for AFK (optional)")]
    reason: Option<String>,

    #[arg(short = 'C', long, help = "Disable color output")]
    without_color: bool,

    #[arg(short = 'T', long, help = "Disable timestamp output")]
    without_timestamp: bool,

    #[arg(short, long, help = "Set the speed of the animation", value_parser = ["fast", "normal", "slow"], default_value = "normal", hide_default_value = true)]
    speed: String,
}

fn get_styles() -> clap::builder::Styles {
    clap::builder::Styles::default()
        .header(styling::AnsiColor::Yellow.on_default().bold().underline())
        .usage(styling::AnsiColor::Yellow.on_default().bold().underline())
        .literal(styling::AnsiColor::Green.on_default())
}
