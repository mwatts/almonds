use askama::Template;

#[derive(Template)]
#[template(path = "password_reset.html")]
pub struct PasswordResetTemplate<'a> {
    pub otp: &'a str,
}
