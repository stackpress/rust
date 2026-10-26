//! One closed rich-content policy shared by native rendering and actual WASM.
//!
//! Sanitization is mandatory before HTML insertion. Scripts, styles, SVG, MathML,
//! frames, forms, images and relative URLs are excluded. Only explicit HTTP/HTTPS
//! links survive. CSP is additional protection, never a substitute for this policy.

use leptos::prelude::*;
use std::collections::{HashMap, HashSet};

/// The initial fixed sanitizer revision; changes require corpus/browser review.
pub const POLICY_REVISION: u32 = 1;
/// Maximum untrusted input bytes, before Markdown parsing or HTML sanitization.
pub const MAX_INPUT_BYTES: usize = 64 * 1024;
/// Maximum sanitized output bytes, including parser expansion.
pub const MAX_OUTPUT_BYTES: usize = 256 * 1024;

/// Budget rejection without echoing hostile content.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RichError {
    /// Untrusted input exceeded its admission limit.
    InputLimit,
    /// Sanitized output exceeded its publication limit.
    OutputLimit,
}
impl std::fmt::Display for RichError {
    /// Keep public diagnostics free of the untrusted input.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::InputLimit => "Rich input too large",
            Self::OutputLimit => "Rich output too large",
        })
    }
}
impl std::error::Error for RichError {}

/// Checked HTML; no public unchecked constructor or deserializer exists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SafeHtml(String);

impl SafeHtml {
    /// Sanitize HTML using the fixed tag/attribute/scheme policy.
    ///
    /// # Errors
    /// Rejects input/output budget excess. It never accepts trusted raw HTML.
    pub fn html(input: &str) -> Result<Self, RichError> {
        Self::with_limits(input, MAX_INPUT_BYTES, MAX_OUTPUT_BYTES)
    }

    /// Parse CommonMark, then apply the identical HTML sanitizer.
    ///
    /// # Errors
    /// Rejects input/output budget excess; Markdown parsing does not grant HTML trust.
    pub fn markdown(input: &str) -> Result<Self, RichError> {
        if input.len() > MAX_INPUT_BYTES {
            return Err(RichError::InputLimit);
        }
        let mut html = String::new();
        pulldown_cmark::html::push_html(&mut html, pulldown_cmark::Parser::new(input));
        if html.len() > MAX_OUTPUT_BYTES {
            return Err(RichError::OutputLimit);
        }
        Self::sanitize(&html, MAX_OUTPUT_BYTES)
    }

    /// Sanitize under lowerable byte ceilings for application policy and boundary tests.
    ///
    /// # Errors
    /// Rejects a requested ceiling above the fixed maximum, or content exceeding it.
    pub fn with_limits(
        input: &str,
        input_limit: usize,
        output_limit: usize,
    ) -> Result<Self, RichError> {
        if input_limit > MAX_INPUT_BYTES || input.len() > input_limit {
            return Err(RichError::InputLimit);
        }
        if output_limit > MAX_OUTPUT_BYTES {
            return Err(RichError::OutputLimit);
        }
        Self::sanitize(input, output_limit)
    }

    /// Apply the one closed policy after the caller has admitted bounded input.
    fn sanitize(input: &str, output_limit: usize) -> Result<Self, RichError> {
        // Replace every configurable policy surface, rather than inheriting defaults.
        let tags = HashSet::from([
            "p",
            "a",
            "strong",
            "em",
            "ul",
            "ol",
            "li",
            "code",
            "pre",
            "blockquote",
            "br",
        ]);
        let attributes = HashMap::from([("a", HashSet::from(["href", "title"]))]);
        let output = ammonia::Builder::new()
            .tags(tags)
            .tag_attributes(attributes)
            .generic_attributes(HashSet::new())
            .url_schemes(HashSet::from(["http", "https"]))
            .url_relative(ammonia::UrlRelative::Deny)
            .link_rel(Some("noopener noreferrer"))
            .strip_comments(true)
            .clean(input)
            .to_string();
        if output.len() > output_limit {
            return Err(RichError::OutputLimit);
        }
        Ok(Self(output))
    }

    /// Borrow only the sanitized output, for rendering or parity verification.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The only rich insertion boundary accepts the private checked type.
#[component]
pub fn RichContent(
    /// Sanitized value, constructed identically on SSR/hydrate/CSR.
    content: SafeHtml,
) -> impl IntoView {
    view! { <div class="sp-rich" inner_html=content.0></div> }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Malformed markup, executable URLs and disallowed elements remain inert and idempotent.
    #[test]
    fn closed_policy_and_budgets() {
        for input in [
            "<script>alert(1)</script><p onclick='x()'>safe</p>",
            "<svg><g onload='x()'><a xlink:href='javascript:x'>x</a></g></svg>",
            "<math><mtext><table><mglyph><style><!--</style><img title='--><img src=x onerror=x>'>",
            "<a href='jav&#x61;script:x'>x</a><a href='//evil.invalid'>x</a><a href='/local'>x</a>",
            "<form><input autofocus><iframe src='https://evil.invalid'></iframe><img src='https://evil.invalid'></form>",
            "<a href='data:text/html,x'>x</a><style>body{display:none}</style>",
        ] {
            let safe = SafeHtml::html(input).unwrap();
            assert_eq!(SafeHtml::html(safe.as_str()).unwrap(), safe);
            for forbidden in [
                "<script", "<style", "<svg", "<math", "<form", "<input", "<iframe", "<img",
                "href=", "onclick", "onerror",
            ] {
                assert!(
                    !safe.as_str().contains(forbidden),
                    "{}: {forbidden}",
                    safe.as_str()
                );
            }
        }
        let safe =
            SafeHtml::markdown("**safe** <a href='https://example.invalid'>link</a>").unwrap();
        assert!(safe.as_str().contains("<strong>safe</strong>"));
        assert!(safe.as_str().contains("noopener noreferrer"));
        assert_eq!(
            SafeHtml::with_limits("long", 1, 8),
            Err(RichError::InputLimit)
        );
        assert_eq!(
            SafeHtml::with_limits("&", 1, 1),
            Err(RichError::OutputLimit)
        );
        assert_eq!(
            SafeHtml::with_limits("", 0, MAX_OUTPUT_BYTES + 1),
            Err(RichError::OutputLimit)
        );
        assert!(SafeHtml::html("").unwrap().as_str().is_empty());
    }
}
