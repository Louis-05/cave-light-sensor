use eframe::egui;


#[derive(PartialEq,Clone,Copy)]
pub enum NoDataCommand {
    Ping,
    SetSensorId,
    GetEverything,
    GetEverythingAndSave,
    ResetSensor,
    GetTime,
    SyncTime,
}

const COMMAND_LIST: [NoDataCommand; 7] = [
    NoDataCommand::Ping,
    NoDataCommand::SetSensorId,
    NoDataCommand::GetEverything,
    NoDataCommand::GetEverythingAndSave,
    NoDataCommand::ResetSensor,
    NoDataCommand::GetTime,
    NoDataCommand::SyncTime,
];

#[derive(Clone)]
pub struct CommandSelector {
    pub selected_command: NoDataCommand,
}



impl Default for CommandSelector {
    fn default() -> Self {
        Self {
            selected_command: NoDataCommand::Ping,
        }
    }
}

impl CommandSelector {
    pub fn show(&mut self, ui: &mut egui::Ui) -> egui::Response {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                for command in COMMAND_LIST {
                    ui.radio_value(&mut self.selected_command, command, command.to_string());
                }
            });
        })
        .response
    }
}

impl NoDataCommand {
    pub fn to_string(&self) -> String {
        match self {
            NoDataCommand::Ping => "Ping".to_string(),
            NoDataCommand::SetSensorId => "SetSensorId".to_string(),
            NoDataCommand::GetEverything => "GetEverything".to_string(),
            NoDataCommand::GetEverythingAndSave => "GetEverythingAndSave".to_string(),
            NoDataCommand::ResetSensor => "ResetSensor".to_string(),
            NoDataCommand::GetTime => "GetTime".to_string(),
            NoDataCommand::SyncTime => "SyncTime".to_string(),
        }
    }
}