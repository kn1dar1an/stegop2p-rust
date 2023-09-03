extern crate libc;
extern crate ncurses;

use libc::{ c_char, c_int };
use ncurses::WINDOW;

#[derive(Debug)]
pub struct Ui {
    window: WINDOW,
    size_x: c_int,
    size_y: c_int,
}

impl Ui {
    pub fn new() -> Self {
        let ui: Ui = Self {
            window: ncurses::initscr(),
            size_x: ncurses::COLS(),
            size_y: ncurses::LINES(),
        };

        ncurses::cbreak(); //character-by-character input
        ncurses::echo(); //echo characters back
        ncurses::keypad(ncurses::stdscr(), true); //use function-key mapping
        ui.greet();

        ui
    }
    
    pub fn move_add_string(&self, x: c_int, y: c_int, s: &str) {
        ncurses::mvwaddstr(self.window, y, x, s);
        ncurses::wrefresh(self.window);
    }

    fn greet(&self) {
        self.move_add_string(0, self.size_y - 1, "Welcome to Stegop2p! Press any key to continue...");
        ncurses::wgetch(self.window);
        ncurses::clear();
        self.move_add_string(0, self.size_y - 1,  "Message> ");
    }
}

pub fn create_ui() -> Ui {
    let ui: Ui = Ui::new();
    ui
}

pub fn destroy_ui() {
    ncurses::endwin();
}