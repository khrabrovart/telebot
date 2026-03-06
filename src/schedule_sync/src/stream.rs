pub enum StreamAction {
    Insert,
    Modify,
    Remove,
    Unknown,
}

impl StreamAction {
    pub fn from_event_name(event_name: &str) -> Self {
        match event_name {
            "INSERT" => StreamAction::Insert,
            "MODIFY" => StreamAction::Modify,
            "REMOVE" => StreamAction::Remove,
            _ => StreamAction::Unknown,
        }
    }
}
