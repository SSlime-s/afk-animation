use std::{
    thread,
    time::{self},
};

const BAK_AA: &'static str = r"
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
const AFK_AA: &'static str = r"
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
struct AfkAA {
    idx: usize,
    interval: usize,
    afk_verticals: Vec<Vec<char>>,
}
impl AfkAA {
    fn new(interval: usize) -> AfkAA {
        let lines = AFK_AA
            .trim_start_matches("\n")
            .lines()
            .map(|x| x.chars().collect())
            .collect::<Vec<Vec<char>>>();
        let verticals = (0..(&lines).into_iter().map(|x| x.len()).max().unwrap())
            .map(|i| {
                (0..lines.len())
                    .map(|j| lines.get(j)?.get(i))
                    .map(|x| if let Some(&y) = x { y } else { ' ' })
                    .collect()
            })
            .collect::<Vec<Vec<char>>>();
        AfkAA {
            idx: 0,
            interval,
            afk_verticals: verticals,
        }
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

struct Lines {
    lines: Vec<Vec<char>>,
    afk_aa: AfkAA,
}
impl Lines {
    fn new() -> Self {
        let afk_aa = AfkAA::new(20);
        Self {
            lines: vec![Vec::new(); afk_aa.height()],
            afk_aa,
        }
    }

    fn update(&mut self, limit: usize) -> Vec<String> {
        assert!(limit > 0);
        while self.add_vertical_line() < limit {}
        while self
            .remove_first_vertical_line()
            .expect("Failed to update AFK")
            >= limit
        {}
        self.into_strings()
    }

    fn add_vertical_line(&mut self) -> usize {
        let nxt = self.afk_aa.next().unwrap();
        self.lines
            .iter_mut()
            .zip(nxt.into_iter())
            .for_each(|(base, new)| base.push(new));
        self.lines[0].len()
    }

    fn remove_first_vertical_line(&mut self) -> Result<usize, ()> {
        if self.lines[0].len() == 0 {
            Err(())
        } else {
            self.lines.iter_mut().for_each(|x| {
                x.remove(0);
            });
            Ok(self.lines[0].len())
        }
    }

    fn into_strings(&self) -> Vec<String> {
        self.lines
            .clone()
            .into_iter()
            .map(|x| x.iter().collect())
            .collect()
    }
}

fn main() {
    let saved_terattr = get_terattr_from_os();

    {
        let mut termattr = saved_terattr;
        termattr.c_lflag = termattr.c_lflag & !(libc::ICANON | libc::ECHO);
        termattr.c_cc[libc::VMIN] = 0;
        set_terattr(&termattr);
    }
    ready_to_key_input();

    let mut buf: [libc::c_char; 1] = [0; 1];
    let ptr = &mut buf;

    let mut lines = Lines::new();
    let mut last_width = get_terminal_width().expect("Failed to get terminal Width");
    for x in lines.update(last_width) {
        println!("{}", x);
    }
    loop {
        let r = unsafe { libc::read(0, ptr.as_ptr() as *mut libc::c_void, 1) };
        if r > 0 {
            break;
        }
        thread::sleep(time::Duration::from_millis(100));

        let width = get_terminal_width().expect("Failed to get terminal Width");
        last_width = width;

        print!("\x1b[{}F", lines.afk_aa.height());
        for x in lines.update(width) {
            println!("{}", x);
        }
    }

    print!("\x1b[{}F", lines.afk_aa.height());
    for line in BAK_AA.trim_start_matches("\n").lines() {
        // "\x1b[K" == ESC[K : 行末までをクリア (空白埋めすると狭くしたときに描画が終わる)
        println!("\x1b[K{}", &line[0..last_width.min(line.len())]);
    }

    set_terattr(&saved_terattr);
}

fn get_terminal_width() -> Result<usize, ()> {
    std::process::Command::new("tput")
        .arg("cols")
        .output()
        .map_err(|e| {
            eprintln!("{}", e);
            ()
        })
        .and_then(|x| {
            std::str::from_utf8(&x.stdout)
                .map_err(|e| {
                    eprintln!("{}", e);
                    ()
                })
                .and_then(|x| {
                    x.trim().parse::<usize>().map_err(|e| {
                        eprintln!("{}", e);
                        ()
                    })
                })
        })
}

fn get_terattr_from_os() -> libc::termios {
    let mut attr = libc::termios {
        c_iflag: 0,
        c_oflag: 0,
        c_cflag: 0,
        c_lflag: 0,
        c_cc: [0u8; 32],
        c_ispeed: 0,
        c_ospeed: 0,
        c_line: 0,
    };
    unsafe {
        libc::tcgetattr(0, &mut attr);
    }
    attr
}

fn set_terattr(attr: &libc::termios) {
    unsafe {
        libc::tcsetattr(0, libc::TCSANOW, attr);
    }
}

fn ready_to_key_input() {
    unsafe {
        libc::fcntl(libc::F_SETFL, libc::O_NONBLOCK);
    }
}
