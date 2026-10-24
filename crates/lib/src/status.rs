//! Application status vocabulary; event dispatch outcomes use a separate enum.
//!
//! ```
//! use stackpress_lib::Status;
//! assert_eq!(Status::from_code(419).message, "Page Expired");
//! assert_eq!(Status::from_code(9999).code, 9999);
//! ```

use serde::Serialize;
use std::borrow::Cow;

/// An application code/message pair, independent of native transport legality.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct Status {
    /// Application numeric code; unknown values are retained.
    pub code: u16,
    /// Known static label or an explicitly supplied owned message.
    pub message: Cow<'static, str>,
}

impl Status {
    /// Application 100: Continue.
    pub const CONTINUE: Self = Self {
        code: 100,
        message: Cow::Borrowed("Continue"),
    };
    /// Application 102: Processing.
    pub const PROCESSING: Self = Self {
        code: 102,
        message: Cow::Borrowed("Processing"),
    };
    /// Application 200: OK.
    pub const OK: Self = Self {
        code: 200,
        message: Cow::Borrowed("OK"),
    };
    /// Application 201: Created.
    pub const CREATED: Self = Self {
        code: 201,
        message: Cow::Borrowed("Created"),
    };
    /// Application 202: Accepted.
    pub const ACCEPTED: Self = Self {
        code: 202,
        message: Cow::Borrowed("Accepted"),
    };
    /// Application 204: No Content.
    pub const EMPTY: Self = Self {
        code: 204,
        message: Cow::Borrowed("No Content"),
    };
    /// Application 205: Reset Content.
    pub const RESET: Self = Self {
        code: 205,
        message: Cow::Borrowed("Reset Content"),
    };
    /// Application 206: Partial Content.
    pub const PARTIAL: Self = Self {
        code: 206,
        message: Cow::Borrowed("Partial Content"),
    };
    /// Application 301: Moved Permanently.
    pub const MOVED: Self = Self {
        code: 301,
        message: Cow::Borrowed("Moved Permanently"),
    };
    /// Application 302: Found.
    pub const FOUND: Self = Self {
        code: 302,
        message: Cow::Borrowed("Found"),
    };
    /// Application 303: See Other.
    pub const REDIRECT: Self = Self {
        code: 303,
        message: Cow::Borrowed("See Other"),
    };
    /// Application 304: Not Modified.
    pub const CACHE: Self = Self {
        code: 304,
        message: Cow::Borrowed("Not Modified"),
    };
    /// Application 307: Temporary Redirect.
    pub const TEMPORARY: Self = Self {
        code: 307,
        message: Cow::Borrowed("Temporary Redirect"),
    };
    /// Application 308: Permanent Redirect.
    pub const PERMANENT: Self = Self {
        code: 308,
        message: Cow::Borrowed("Permanent Redirect"),
    };
    /// Application 309: Aborted.
    pub const ABORT: Self = Self {
        code: 309,
        message: Cow::Borrowed("Aborted"),
    };
    /// Application 400: Bad Request.
    pub const BAD_REQUEST: Self = Self {
        code: 400,
        message: Cow::Borrowed("Bad Request"),
    };
    /// Application 401: Unauthorized.
    pub const UNAUTHORIZED: Self = Self {
        code: 401,
        message: Cow::Borrowed("Unauthorized"),
    };
    /// Application 403: Forbidden.
    pub const FORBIDDEN: Self = Self {
        code: 403,
        message: Cow::Borrowed("Forbidden"),
    };
    /// Application 404: Not Found.
    pub const NOT_FOUND: Self = Self {
        code: 404,
        message: Cow::Borrowed("Not Found"),
    };
    /// Application 405: Method Not Allowed.
    pub const BAD_METHOD: Self = Self {
        code: 405,
        message: Cow::Borrowed("Method Not Allowed"),
    };
    /// Application 406: Not Acceptable.
    pub const NOT_ACCEPTABLE: Self = Self {
        code: 406,
        message: Cow::Borrowed("Not Acceptable"),
    };
    /// Application 408: Request Timeout.
    pub const REQUEST_TIMEOUT: Self = Self {
        code: 408,
        message: Cow::Borrowed("Request Timeout"),
    };
    /// Application 409: Conflict.
    pub const CONFLICT: Self = Self {
        code: 409,
        message: Cow::Borrowed("Conflict"),
    };
    /// Application 410: Gone.
    pub const GONE: Self = Self {
        code: 410,
        message: Cow::Borrowed("Gone"),
    };
    /// Application 411: Length Required.
    pub const LENGTH_REQUIRED: Self = Self {
        code: 411,
        message: Cow::Borrowed("Length Required"),
    };
    /// Application 413: Payload Too Large.
    pub const TOO_LARGE: Self = Self {
        code: 413,
        message: Cow::Borrowed("Payload Too Large"),
    };
    /// Application 414: URI Too Long.
    pub const TOO_LONG: Self = Self {
        code: 414,
        message: Cow::Borrowed("URI Too Long"),
    };
    /// Application 415: Unsupported Media Type.
    pub const UNSUPPORTED_TYPE: Self = Self {
        code: 415,
        message: Cow::Borrowed("Unsupported Media Type"),
    };
    /// Application 416: Range Not Satisfiable.
    pub const BAD_RANGE: Self = Self {
        code: 416,
        message: Cow::Borrowed("Range Not Satisfiable"),
    };
    /// Application 417: Expectation Failed.
    pub const BAD_EXPECTATION: Self = Self {
        code: 417,
        message: Cow::Borrowed("Expectation Failed"),
    };
    /// Application 421: Misdirected Request.
    pub const MISDIRECTED: Self = Self {
        code: 421,
        message: Cow::Borrowed("Misdirected Request"),
    };
    /// Application 422: Unprocessable Content.
    pub const UNPROCESSABLE: Self = Self {
        code: 422,
        message: Cow::Borrowed("Unprocessable Content"),
    };
    /// Application 423: Locked.
    pub const LOCKED: Self = Self {
        code: 423,
        message: Cow::Borrowed("Locked"),
    };
    /// Application 424: Failed Dependency.
    pub const BAD_DEPENDENCY: Self = Self {
        code: 424,
        message: Cow::Borrowed("Failed Dependency"),
    };
    /// Application 426: Upgrade Required.
    pub const UPGRADE_REQUIRED: Self = Self {
        code: 426,
        message: Cow::Borrowed("Upgrade Required"),
    };
    /// Application 428: Precondition Required.
    pub const BAD_PRECONDITION: Self = Self {
        code: 428,
        message: Cow::Borrowed("Precondition Required"),
    };
    /// Application 429: Too Many Requests.
    pub const TOO_MANY: Self = Self {
        code: 429,
        message: Cow::Borrowed("Too Many Requests"),
    };
    /// Application 431: Request Header Fields Too Large.
    pub const HEADER_TOO_LARGE: Self = Self {
        code: 431,
        message: Cow::Borrowed("Request Header Fields Too Large"),
    };
    /// Application 451: Unavailable For Legal Reasons.
    pub const LEGAL_REASONS: Self = Self {
        code: 451,
        message: Cow::Borrowed("Unavailable For Legal Reasons"),
    };
    /// Application 419: Page Expired.
    pub const PAGE_EXPIRED: Self = Self {
        code: 419,
        message: Cow::Borrowed("Page Expired"),
    };
    /// Application 500: Internal Server Error.
    pub const ERROR: Self = Self {
        code: 500,
        message: Cow::Borrowed("Internal Server Error"),
    };
    /// Application 501: Not Implemented.
    pub const NOT_IMPLEMENTED: Self = Self {
        code: 501,
        message: Cow::Borrowed("Not Implemented"),
    };
    /// Application 502: Bad Gateway.
    pub const BAD_GATEWAY: Self = Self {
        code: 502,
        message: Cow::Borrowed("Bad Gateway"),
    };
    /// Application 503: Service Unavailable.
    pub const UNAVAILABLE: Self = Self {
        code: 503,
        message: Cow::Borrowed("Service Unavailable"),
    };
    /// Application 504: Gateway Timeout.
    pub const RESPONSE_TIMEOUT: Self = Self {
        code: 504,
        message: Cow::Borrowed("Gateway Timeout"),
    };
    /// Application 505: HTTP Version Not Supported.
    pub const BAD_VERSION: Self = Self {
        code: 505,
        message: Cow::Borrowed("HTTP Version Not Supported"),
    };
    /// Application 507: Insufficient Storage.
    pub const INSUFFICIENT_STORAGE: Self = Self {
        code: 507,
        message: Cow::Borrowed("Insufficient Storage"),
    };
    /// Application 508: Loop Detected.
    pub const INFINITE_LOOP: Self = Self {
        code: 508,
        message: Cow::Borrowed("Loop Detected"),
    };
    /// Application 511: Network Authentication Required.
    pub const NETWORK_AUTHENTICATION_REQUIRED: Self = Self {
        code: 511,
        message: Cow::Borrowed("Network Authentication Required"),
    };
    /// Construct a custom application status, without claiming it is HTTP-valid.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::Status;
    /// let status = Status::new(700, "Custom application code");
    /// assert_eq!(status.code, 700);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn new(code: u16, message: impl Into<String>) -> Self {
        // Own a custom label when requested; the application code is not constrained to native HTTP
        // statuses.
        Self {
            code,
            message: Cow::Owned(message.into()),
        }
    }
    /// Retain a code and use its known label or Unknown.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::Status;
    /// assert_eq!(Status::from_code(9999).message, "Unknown");
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn from_code(code: u16) -> Self {
        // Known entries reuse static labels; unknown codes retain their numeric value with an
        // explicit Unknown label.
        Self::lookup(code).cloned().unwrap_or(Self {
            code,
            message: Cow::Borrowed("Unknown"),
        })
    }
    /// Look up a known code without manufacturing a fallback entry.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::Status;
    /// assert_eq!(Status::lookup(419).unwrap().message, "Page Expired");
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn lookup(code: u16) -> Option<&'static Self> {
        // Return None for unknown codes instead of manufacturing a catalog member.
        CODES.iter().find(|status| status.code == code)
    }
    /// Look up an exact known status message.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stackpress_lib::Status;
    /// assert_eq!(Status::find("OK").unwrap().code, 200);
    /// # Ok::<(), stackpress_lib::Error>(())
    /// ```
    pub fn find(message: &str) -> Option<&'static Self> {
        // Message lookup is exact; it does not normalize or guess an application status.
        CODES.iter().find(|status| status.message == message)
    }
}

/// All source-observed status entries, including legacy 309 and custom 419.
pub const CODES: &[Status] = &[
    Status::CONTINUE,
    Status::PROCESSING,
    Status::OK,
    Status::CREATED,
    Status::ACCEPTED,
    Status::EMPTY,
    Status::RESET,
    Status::PARTIAL,
    Status::MOVED,
    Status::FOUND,
    Status::REDIRECT,
    Status::CACHE,
    Status::TEMPORARY,
    Status::PERMANENT,
    Status::ABORT,
    Status::BAD_REQUEST,
    Status::UNAUTHORIZED,
    Status::FORBIDDEN,
    Status::NOT_FOUND,
    Status::BAD_METHOD,
    Status::NOT_ACCEPTABLE,
    Status::REQUEST_TIMEOUT,
    Status::CONFLICT,
    Status::GONE,
    Status::LENGTH_REQUIRED,
    Status::TOO_LARGE,
    Status::TOO_LONG,
    Status::UNSUPPORTED_TYPE,
    Status::BAD_RANGE,
    Status::BAD_EXPECTATION,
    Status::MISDIRECTED,
    Status::UNPROCESSABLE,
    Status::LOCKED,
    Status::BAD_DEPENDENCY,
    Status::UPGRADE_REQUIRED,
    Status::BAD_PRECONDITION,
    Status::TOO_MANY,
    Status::HEADER_TOO_LARGE,
    Status::LEGAL_REASONS,
    Status::PAGE_EXPIRED,
    Status::ERROR,
    Status::NOT_IMPLEMENTED,
    Status::BAD_GATEWAY,
    Status::UNAVAILABLE,
    Status::RESPONSE_TIMEOUT,
    Status::BAD_VERSION,
    Status::INSUFFICIENT_STORAGE,
    Status::INFINITE_LOOP,
    Status::NETWORK_AUTHENTICATION_REQUIRED,
];
