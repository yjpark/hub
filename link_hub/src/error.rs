pub struct ErrorMessage();

impl ErrorMessage {
    pub const UNDER_CONSTRUCTION: &'static str = "UNDER_CONSTRUCTION";
    pub const INVALID_APP_ID: &'static str = "INVALID_APP_ID";
    pub const INVALID_SESSION_ID: &'static str = "INVALID_SESSION_ID";
    pub const ALREADY_AUTHENTICATED: &'static str = "ALREADY_AUTHENTICATED";
    pub const INVALID_LINK_ID: &'static str = "INVALID_LINK_ID";
}