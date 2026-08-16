use crate::errors::service_error::ServiceError;

pub struct EmailMessage {
    pub from_address: String,
    pub from_name: String,
    pub to_address: String,
    pub to_name: String,
    pub subject: String,
    pub html_body: String,
}

pub trait EmailSender: Send + Sync {
    fn send_email(&self, message: EmailMessage) -> Result<(), ServiceError>;
}
