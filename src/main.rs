mod command;
mod logic;

use anyhow::{ensure, Context as _, Result};
use crossterm::{
    cursor::{Hide, Show},
    execute,
    style::Print,
    terminal::{Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use rand::Rng;
use std::{
    collections::VecDeque,
    io::{stdout, Write},
    thread,
    time::{self},
};

use crate::logic::terminal::get_terminal_width;

const BAK_AA: &str = r"
 _________                       __    ___ 
|   ____  \          /\         |  |  /  / 
|  |    \  |        /  \        |  | /  /  
|  |____/  |       /    \       |  |/  /   
|         /       /  /\  \      |  '  /    
|   ____  \      /  /__\  \     |  .  \    
|  |    \  |    /  ______  \    |  |\  \   
|  |____/  |   /  /      \  \   |  | \  \  
|_________/   /__/        \__\  |__|  \__\ 
";
const AFK_AA: &str = r"
                ______________    __    ___
       /\      |__    ______  |  |  |  /  /
      /  \        |  |      |_|  |  | /  / 
     /    \       |  |    _      |  |/  /  
    /  /\  \      |  |___| |     |  '  /   
   /  /__\  \     |   _____|     |  .  \   
  /  ______  \    |  |           |  |\  \  
 /  /      \  \   |  |           |  | \  \ 
/__/        \__\  |__|           |__|  \__\
                                           ";

const DEFAULT_COLOR: &str = "\x1b[0m";

struct AfkAA {
    idx: usize,
    interval: usize,
    afk_verticals: Vec<Vec<char>>,
}
impl AfkAA {
    fn new(interval: usize) -> Result<Self> {
        let lines = AFK_AA
            .trim_start_matches('\n')
            .lines()
            .map(|x| x.chars().collect())
            .collect::<Vec<Vec<char>>>();
        let verticals = (0..lines
            .iter()
            .map(|x| x.len())
            .max()
            .context("Failed to get max line length")?)
            .map(|i| {
                (0..lines.len())
                    .map(|j| lines.get(j)?.get(i))
                    .map(|x| if let Some(&y) = x { y } else { ' ' })
                    .collect()
            })
            .collect::<Vec<Vec<char>>>();
        Ok(Self {
            idx: 0,
            interval,
            afk_verticals: verticals,
        })
    }

    fn height(&self) -> usize {
        self.afk_verticals[0].len()
    }
}
impl Iterator for AfkAA {
    type Item = Vec<char>;
    fn next(&mut self) -> Option<Vec<char>> {
        let ret = if self.idx < self.afk_verticals.len() {
            self.afk_verticals[self.idx].clone()
        } else {
            vec![' '; self.afk_verticals[0].len()]
        };
        self.idx += 1;
        if self.afk_verticals.len() + self.interval == self.idx {
            self.idx = 0;
        }
        Some(ret)
    }
}

const COLOR_MIN: u8 = 30;
const COLOR_MAX: u8 = 200;
const COLOR_STEP: u8 = 5;

const _: () = {
    assert!(COLOR_MIN < COLOR_MAX);
    assert!((COLOR_MAX - COLOR_MIN) % COLOR_STEP == 0);
};

struct Colorizer {
    rgb: Vec<u8>,
    now_inclement: usize,
}
impl Colorizer {
    fn new() -> Self {
        Self {
            rgb: vec![COLOR_MIN, COLOR_MAX, COLOR_MAX],
            now_inclement: 0,
        }
    }

    fn to_ansi_color(&self) -> String {
        assert_eq!(self.rgb.len(), 3);
        format!("\x1b[38;2;{};{};{}m", self.rgb[0], self.rgb[1], self.rgb[2])
    }
}
impl Iterator for Colorizer {
    type Item = String;
    fn next(&mut self) -> Option<Self::Item> {
        assert_eq!(self.rgb.len(), 3);
        assert!(self.now_inclement < 3);
        if self.rgb[self.now_inclement] == COLOR_MAX {
            self.now_inclement += 1;
            self.now_inclement %= 3;
        }
        self.rgb[self.now_inclement] += COLOR_STEP;
        self.rgb[(self.now_inclement + 1) % 3] -= COLOR_STEP;
        Some(self.to_ansi_color())
    }
}

struct Lines {
    lines: Vec<VecDeque<char>>,
    colors: VecDeque<String>,
    afk_aa: AfkAA,
    colorizer: Colorizer,
    colored: bool,
}
impl Lines {
    fn new(colored: bool) -> Result<Self> {
        let afk_aa = AfkAA::new(20)?;
        Ok(Self {
            lines: vec![VecDeque::new(); afk_aa.height()],
            afk_aa,
            colors: VecDeque::new(),
            colorizer: Colorizer::new(),
            colored,
        })
    }

    fn height(&self) -> usize {
        self.lines.len()
    }

    fn update(&mut self, limit: usize) -> Result<Vec<String>> {
        assert!(limit > 0);
        while self.add_vertical_line()? < limit {}
        while self.remove_first_vertical_line()? >= limit {}

        Ok(self.to_strings())
    }

    fn add_vertical_line(&mut self) -> Result<usize> {
        let nxt = self
            .afk_aa
            .next()
            .context("Failed to get next vertical line")?;
        self.lines
            .iter_mut()
            .zip(nxt)
            .for_each(|(base, new)| base.push_back(new));
        if self.colored {
            self.colors.extend(self.colorizer.by_ref().take(8));
            self.colors = self.colors.split_off(7);
        }
        Ok(self.lines[0].len())
    }

    fn remove_first_vertical_line(&mut self) -> Result<usize> {
        ensure!(
            !self.lines[0].is_empty(),
            "Failed to remove first vertical line"
        );

        self.lines.iter_mut().for_each(|line| {
            line.pop_front();
        });
        if self.colored {
            self.colors.pop_front();
        }
        Ok(self.lines[0].len())
    }

    fn to_strings(&self) -> Vec<String> {
        assert_eq!(self.lines.first().unwrap().len(), self.colors.len());
        self.lines
            .clone()
            .into_iter()
            .map(|line| {
                if self.colored {
                    let colors = self.colors.clone();
                    let elements = colors.into_iter().zip(line);
                    let mut colored_line = elements
                        .map(|(color, ch)| {
                            if ch == ' ' {
                                ch.to_string()
                            } else {
                                format!("{}{}", color, ch)
                            }
                        })
                        .collect::<String>();
                    colored_line.push_str(DEFAULT_COLOR);
                    colored_line
                } else {
                    line.into_iter().collect::<String>()
                }
            })
            .collect()
    }
}

fn main() -> Result<()> {
    let config = crate::command::Config::new();
    let mut key_manager = crate::logic::terminal::KeyManager::new()?;
    let mut timer = crate::logic::timer::Timer::start();

    // disable fold back
    print!("\x1b[?7l");

    execute!(stdout(), Hide, EnterAlternateScreen)?;

    let mut lines = Lines::new(config.colored)?;
    {
        let width = get_terminal_width()?;
        println!("{}", lines.update(width)?.join("\n"));
    }

    loop {
        if key_manager.check() {
            break;
        }
        thread::sleep(time::Duration::from_millis(config.fps));

        let width = get_terminal_width()?;

        print!(
            "\x1b[{}F",
            lines.height() + if config.is_exist_footer() { 1 } else { 0 }
        );
        println!("{}", lines.update(width)?.join("\n"));
        if let Some(message) = generate_footer_message(
            Some(&timer).filter(|_| config.show_timestamp),
            &config.reason,
        ) {
            println!("{}", message);
        }
    }
    timer.finish();

    execute!(stdout(), LeaveAlternateScreen)?;

    print_bak(&config, &timer)?;

    // \x1b[?7h -> enable fold back
    print!("\x1b[?7h");
    execute!(stdout(), Show)?;
    if config.is_exist_footer() {
        // clear line
        // print!("\n\x1b[K");
        execute!(stdout(), Print("\n"), Clear(ClearType::CurrentLine))?;
    }
    std::io::stdout().flush()?;

    Ok(())
}

fn print_bak(config: &crate::command::Config, timer: &crate::logic::timer::Timer) -> Result<()> {
    let colorizer = Colorizer::new();
    let random_skip: usize =
        rand::thread_rng().gen_range(0..(COLOR_MAX - COLOR_MIN) / COLOR_STEP * 3) as usize;
    let colors = colorizer
        .skip(random_skip)
        .take(
            BAK_AA
                .trim_start_matches('\n')
                .lines()
                .next()
                .context("Failed to get line length")?
                .len(),
        )
        .collect::<Vec<_>>();

    for line in BAK_AA.trim_start_matches('\n').lines() {
        if config.colored {
            let colored_line = colors
                .iter()
                .zip(line.chars())
                .map(|(color, ch)| format!("{}{}", color, ch))
                .collect::<String>();
            println!("{}{}", colored_line, DEFAULT_COLOR);
        } else {
            println!("{}", line);
        }
    }
    if let Some(message) = generate_footer_message(
        Some(timer).filter(|_| config.show_timestamp),
        &config.reason,
    ) {
        print!("{}", message);
    }

    Ok(())
}

fn generate_footer_message(
    timer: Option<&crate::logic::timer::Timer>,
    reason: &Option<String>,
) -> Option<String> {
    let formatted_timer = timer.map(|timer| {
        if timer.is_measuring() {
            format!("left from {}", timer.formatted_start())
        } else {
            format!(
                "\x1b[Kleft from {} to {} ({})",
                timer.formatted_start(),
                timer.formatted_end(),
                timer.formatted_duration(),
            )
        }
    });
    match (formatted_timer, reason) {
        (Some(formatted_timer), Some(reason)) => {
            Some(format!("{} | reason: {}", formatted_timer, reason))
        }
        (Some(formatted_timer), None) => Some(formatted_timer),
        (None, Some(reason)) => Some(format!("reason: {}", reason)),
        (None, None) => None,
    }
}
