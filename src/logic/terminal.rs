use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

pub struct KeyManager {
    saved_terattr: libc::termios,
    buf: [libc::c_char; 1],
    is_ctrl: Arc<AtomicBool>,
}
impl KeyManager {
    pub fn new() -> Self {
        let saved_terattr = Self::get_terattr_from_os();
        let mut termattr = saved_terattr;

        termattr.c_lflag &= !(libc::ICANON | libc::ECHO);
        termattr.c_cc[libc::VMIN] = 0;
        Self::set_terattr(&termattr);

        Self::ready_to_key_input();

        let is_ctrl = Arc::new(AtomicBool::new(false));
        {
            let is_ctrl = is_ctrl.clone();
            ctrlc::set_handler(move || {
                is_ctrl.store(true, Ordering::SeqCst);
            })
            .expect("Failed to set Ctrl-C handler");
        }

        Self {
            saved_terattr,
            buf: [0; 1],
            is_ctrl,
        }
    }

    pub fn check(&mut self) -> bool {
        let input = unsafe { libc::read(libc::STDIN_FILENO, &mut self.buf as *mut _ as *mut _, 1) };
        input > 0 || self.is_ctrl.load(Ordering::SeqCst)
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
}
impl Drop for KeyManager {
    fn drop(&mut self) {
        Self::set_terattr(&self.saved_terattr);
    }
}

pub fn get_terminal_width() -> Result<usize, ()> {
    std::process::Command::new("tput")
        .arg("cols")
        .output()
        .map_err(|_e| ())
        .and_then(|output| {
            std::str::from_utf8(&output.stdout)
                .map_err(|_e| ())
                .and_then(|width_str| width_str.trim().parse::<usize>().map_err(|_e| ()))
        })
}
