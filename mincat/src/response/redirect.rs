use http::{header, HeaderValue, StatusCode};
use mincat_core::response::{IntoResponse, IntoResponseParts, Response};

pub struct Redirect {
    status_code: StatusCode,
    location: HeaderValue,
}

impl Redirect {
    pub fn sse_other(uri: &str) -> Self {
        Self::with_status_code(StatusCode::SEE_OTHER, uri)
    }

    pub fn temporary(uri: &str) -> Self {
        Self::with_status_code(StatusCode::TEMPORARY_REDIRECT, uri)
    }

    pub fn permanent(uri: &str) -> Self {
        Self::with_status_code(StatusCode::PERMANENT_REDIRECT, uri)
    }

    fn with_status_code(status_code: StatusCode, uri: &str) -> Self {
        if !status_code.is_redirection() {
            panic!("status code can't redirection")
        }

        Self {
            status_code,
            location: HeaderValue::try_from(uri).expect("uri isn't a valid header value"),
        }
    }
}

impl IntoResponse for Redirect {
    fn into_response(self) -> Response {
        (self.status_code, [(header::LOCATION, self.location)]).into_response()
    }
}

impl IntoResponseParts for Redirect {
    fn into_response_parts(self, mut response: Response) -> Response {
        *response.status_mut() = self.status_code;
        response
            .headers_mut()
            .extend([(header::LOCATION, self.location)]);
        response
    }
}
