use crate::commands::Command;

pub enum Message {
    Connect{path : String, baud : u32},
    Command(Command),
}