use std::env;

use async_trait::async_trait;
use bcrypt::{hash, verify, DEFAULT_COST};
use lettre::{
    message::header::ContentType, message::Mailbox, transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor,
};

use crate::errors::service_error::ServiceError;

#[derive(Clone)]
pub struct ServiceHelpers {}

impl ServiceHelpers {
    pub fn init() -> Self {
        Self {}
    }
}

async fn dispatch_email(to: &str, subject: &str, body: String) -> Result<(), ServiceError> {
    let host = env::var("SMTP_HOST")
        .map_err(|_| ServiceError::InternalError("SMTP_HOST not configured".into()))?;
    let port: u16 = env::var("SMTP_PORT")
        .unwrap_or_else(|_| "587".into())
        .parse()
        .unwrap_or(587);
    let username = env::var("SMTP_AUTH_USERNAME")
        .map_err(|_| ServiceError::InternalError("SMTP_AUTH_USERNAME not configured".into()))?;
    let password = env::var("SMTP_AUTH_PASSWORD")
        .map_err(|_| ServiceError::InternalError("SMTP_AUTH_PASSWORD not configured".into()))?;

    let from: Mailbox = username
        .parse()
        .map_err(|_| ServiceError::OperationFailed)?;
    let to: Mailbox = to.parse().map_err(|_| ServiceError::OperationFailed)?;

    let email = Message::builder()
        .from(from)
        .to(to)
        .subject(subject)
        .header(ContentType::TEXT_PLAIN)
        .body(body)
        .map_err(|_| ServiceError::OperationFailed)?;

    let creds = Credentials::new(username, password);
    let mailer = AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&host)
        .map_err(|_| ServiceError::OperationFailed)?
        .credentials(creds)
        .port(port)
        .build();

    mailer
        .send(email)
        .await
        .map_err(|_| ServiceError::OperationFailed)?;

    Ok(())
}

#[async_trait]
pub trait ServiceHelpersTrait {
    fn hash_password(&self, raw_password: &str) -> Result<String, ServiceError>;
    fn validate_password(&self, raw_password: &str, hash: &str) -> Result<bool, ServiceError>;

    async fn send_account_confirmation_email(
        &self,
        user_email: &str,
        otp: &str,
    ) -> Result<(), ServiceError>;

    async fn send_forgotten_password_email(
        &self,
        user_email: &str,
        otp: &str,
    ) -> Result<(), ServiceError>;
}

#[async_trait]
impl ServiceHelpersTrait for ServiceHelpers {
    fn hash_password(&self, raw_password: &str) -> Result<String, ServiceError> {
        hash(raw_password, DEFAULT_COST).map_err(|_| ServiceError::OperationFailed)
    }

    fn validate_password(&self, password: &str, hash: &str) -> Result<bool, ServiceError> {
        verify(password, hash).map_err(|_| ServiceError::OperationFailed)
    }

    async fn send_account_confirmation_email(
        &self,
        user_email: &str,
        otp: &str,
    ) -> Result<(), ServiceError> {
        dispatch_email(
            user_email,
            "Verify your Daily account",
            format!(
                "Your verification code is: {otp}\n\nThis code expires in 25 minutes.\n\nIf you did not create an account, please ignore this email."
            ),
        )
        .await
    }

    async fn send_forgotten_password_email(
        &self,
        user_email: &str,
        otp: &str,
    ) -> Result<(), ServiceError> {
        dispatch_email(
            user_email,
            "Reset your Daily password",
            format!(
                "Your password reset code is: {otp}\n\nThis code expires in 25 minutes.\n\nIf you did not request a password reset, please ignore this email."
            ),
        )
        .await
    }
}
