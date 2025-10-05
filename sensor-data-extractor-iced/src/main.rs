use iced::{Application, Settings};
use ui::Ui;

mod ui;
mod serialport;

fn main() -> iced::Result{
    Ui::run(Settings::default())
}
