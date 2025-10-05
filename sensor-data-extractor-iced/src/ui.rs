use std::path::PathBuf;

use iced::{executor, Alignment, Application, Command, Length, Theme};
use iced::widget::{button, column, container, pick_list, row, scrollable, text, vertical_space};
use serial2_tokio::SerialPort;

use crate::serialport::MySerialPort;

#[derive(Default)]
pub struct Ui {
    pub available_serials : Vec<MySerialPort>,
    pub selected_serial : Option<MySerialPort>
}

#[derive(Debug, Clone)]
pub enum Message {
   UpdateSerials,
   ConnectSerial(MySerialPort),
   SerialSelected(MySerialPort)
}

impl Application for Ui {
    type Message = Message;
    type Theme = Theme;
    type Executor = executor::Default;
    type Flags = ();

    fn new(flags: Self::Flags) -> (Self, iced::Command<Self::Message>) {
        (Self {
            available_serials: Vec::new(),
            selected_serial: None,
        }, Command::none())
    }

    fn title(&self) -> String {
        String::from("Sensor data extractor")
    }

    fn subscription(&self) -> iced::Subscription<Self::Message> {
        iced::Subscription::none()
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Message> {
        match message {
            Message::UpdateSerials => self.available_serials = get_serials(),
            _ => ()
        }
        Command::none()
    }

    fn view(&self) -> iced::Element<'_, Self::Message> {
       row![
        column![
            button("Refresh").on_press(Message::UpdateSerials),
            pick_list(self.available_serials.as_ref(),self.selected_serial.clone(),|s|Message::SerialSelected(s)).placeholder("Serial Port"),
            button("Connect"),
            

        ].padding(20)
        .align_items(Alignment::Center)
        .width(Length::Fill)
        


        ].padding(20)
       .align_items(Alignment::Center)
       .into()

    }
}



pub fn get_serials() -> Vec<MySerialPort> {
    SerialPort::available_ports().unwrap_or_else(|_|Vec::new()).iter().map(|p|MySerialPort::new(p)).collect()
}