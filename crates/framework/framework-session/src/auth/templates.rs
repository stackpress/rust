//! Original sign-in template rows and explicit HTML escaping, including plain-text bodies.

/// Rendered source message fields; provider options are assembled by the email process.
#[derive(Clone, Debug, PartialEq, Eq, serde::Serialize)]
pub struct Template {
    /// Original fixed subject.
    pub subject: String,
    /// Plain text with the same preescaped substitutions as the source template engine.
    pub text: String,
    /// HTML with escaped values.
    pub html: String,
}
/// Original sign-in template input; omitted brand and pin have source defaults.
#[derive(Clone, Copy, Debug)]
pub struct SigninEmailTemplateInput<'a> {
    /// Missing brand uses Stackpress; an explicit empty brand stays empty.
    pub brand: Option<&'a str>,
    /// Recipient address, also the greeting fallback.
    pub email: &'a str,
    /// Absolute sign-in link, escaped in both text and HTML rows.
    pub link: &'a str,
    /// Greeting is trimmed before falling back to email.
    pub name: Option<&'a str>,
    /// OTP code; missing uses an empty string. Magic-link rendering ignores it.
    pub pin: Option<&'a str>,
}
/// Render the source one-time-password subject/text/HTML row.
pub fn make_otp_template(input: SigninEmailTemplateInput<'_>) -> Template {
    render(true, input)
}
/// Render the source magic-link subject/text/HTML row.
pub fn make_magic_link_template(input: SigninEmailTemplateInput<'_>) -> Template {
    render(false, input)
}
/// Render a selected source row after applying its defaults once.
fn render(otp: bool, input: SigninEmailTemplateInput<'_>) -> Template {
    let SigninEmailTemplateInput {
        brand,
        email,
        link,
        name,
        pin,
    } = input;
    let brand = brand.unwrap_or("Stackpress");
    let pin = pin.unwrap_or("");
    let name = escape(
        name.map(super::bridge::trim)
            .filter(|name| !name.is_empty())
            .unwrap_or(email),
    );
    let brand = escape(brand);
    let link = escape(link);
    let pin = escape(pin);
    if otp {
        Template {
            subject: "Stackpress Sign In OTP".into(),
            text: format!(
                "Hello {name},\n\nYou requested a one-time password to sign in to {brand}.\n\n----\n\nYour OTP is: {pin}\nOpen {link} to continue signing in.\n\n----\n\nIf you did not request this sign-in, please ignore this email."
            ),
            html: format!(
                "<p>Hello {name},</p>\n<p>You requested a one-time password to sign in to {brand}.</p>\n<hr />\n<p>Your OTP is: <strong>{pin}</strong></p>\n<p>Open <a href=\"{link}\">this sign-in page</a> to continue.</p>\n<hr />\n<p>If you did not request this sign-in, please ignore this email.</p>"
            ),
        }
    } else {
        Template {
            subject: "Stackpress Magic Sign In Link".into(),
            text: format!(
                "Hello {name},\n\nYou requested a magic link to sign in to {brand}.\n\n----\n\nOpen {link} to continue signing in.\n\n----\n\nIf you did not request this sign-in, please ignore this email."
            ),
            html: format!(
                "<p>Hello {name},</p>\n<p>You requested a magic link to sign in to {brand}.</p>\n<hr />\n<p><a href=\"{link}\">Click here to sign in</a></p>\n<hr />\n<p>If you did not request this sign-in, please ignore this email.</p>"
            ),
        }
    }
}
/// Match the source replacement order, so preexisting entities are escaped again.
fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
