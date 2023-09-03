mod ui;

use ui::Ui;

fn main() {
    let window: Ui = ui::create_ui();
    ui::destroy_ui();
}
