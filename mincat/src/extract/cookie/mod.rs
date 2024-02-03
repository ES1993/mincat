use cookie::Key;
use http::{
    header::{COOKIE, SET_COOKIE},
    HeaderMap,
};

#[cfg(feature = "cookie")]
mod normal;
#[cfg(feature = "cookie")]
pub use normal::CookieJar;

#[cfg(feature = "cookie-private")]
mod private;
#[cfg(feature = "cookie-private")]
pub use private::PrivateCookieJar;

#[cfg(feature = "cookie-signed")]
mod signed;
#[cfg(feature = "cookie-signed")]
pub use signed::SignedCookieJar;

pub use cookie::Cookie;

#[derive(Clone)]
pub struct CookieKey(Key);

impl CookieKey {
    fn fix_len_to_64(bytes: Vec<u8>) -> Vec<u8> {
        if bytes.len() > 64 {
            bytes
        } else {
            let mut res = vec![];
            while res.len() < 64 {
                res.extend_from_slice(&bytes);
            }
            res.truncate(64);
            res
        }
    }

    pub fn from(key: &str) -> Self {
        let key = key.as_bytes();
        let key = CookieKey::fix_len_to_64(key.to_vec());
        CookieKey(Key::from(&key))
    }
}

fn cookies_from_request(headers: &HeaderMap) -> impl Iterator<Item = Cookie<'static>> + '_ {
    headers
        .get_all(COOKIE)
        .into_iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|value| value.split(';'))
        .filter_map(|cookie| Cookie::parse_encoded(cookie.to_owned()).ok())
}

fn set_cookies(jar: cookie::CookieJar, headers: &mut HeaderMap) {
    for cookie in jar.delta() {
        if let Ok(header_value) = cookie.encoded().to_string().parse() {
            headers.append(SET_COOKIE, header_value);
        }
    }
}
