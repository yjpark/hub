pub struct ErrorMessage();

pub type LinkErrorMessage = link_hub::error::ErrorMessage;

impl ErrorMessage {
    pub const UNDER_CONSTRUCTION: &'static str = LinkErrorMessage::UNDER_CONSTRUCTION;
    pub const INVALID_APP_ID: &'static str = LinkErrorMessage::INVALID_APP_ID;
    pub const INVALID_SESSION_ID: &'static str = LinkErrorMessage::INVALID_SESSION_ID;
    pub const INVALID_ORD: &'static str = LinkErrorMessage::INVALID_ORD;
    pub const ALREADY_REGISTERED: &'static str = "ALREADY_REGISTERED";
    pub const INVALID_SINK_ID: &'static str = "INVALID_SINK_ID";
    pub const NO_SINK_CONNECTED: &'static str = "NO_SINK_CONNECTED";
    pub const NO_APP_FOUND_BY_ADDRESS: &'static str = "NO_APP_FOUND_BY_ADDRESS";
}
