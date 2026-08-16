use askama::Template;

#[derive(Template)]
#[template(path = "account_confirmation.html")]
pub struct AccountConfirmationTemplate<'a> {
    pub otp: &'a str,
}
