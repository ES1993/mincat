use std::{
    convert::Infallible,
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};

use bytes::{BufMut, Bytes, BytesMut};
use futures_util::{
    ready,
    stream::{BoxStream, Stream},
    Future, StreamExt,
};
use http::{header, HeaderValue, StatusCode};
use hyper::body::Frame;
use mincat_core::{
    body::Body,
    response::{IntoResponse, Response},
};
use pin_project_lite::pin_project;
use tokio::time::Sleep;

pub struct Sse {
    stream: BoxStream<'static, Event>,
    keep_alive: Option<KeepAlive>,
}

impl IntoResponse for Sse {
    fn into_response(self) -> Response {
        (
            StatusCode::OK,
            [
                (
                    header::CONTENT_TYPE,
                    HeaderValue::from_static(mime::TEXT_EVENT_STREAM.as_ref()),
                ),
                (header::CACHE_CONTROL, HeaderValue::from_static("no-cache")),
            ],
            Body::new(SseBody {
                event_stream: self.stream,
                keep_alive: self.keep_alive.map(KeepAliveStream::new),
            }),
        )
            .into_response()
    }
}

impl Sse {
    pub fn new<S>(stream: S) -> Self
    where
        S: Stream<Item = Event> + Send + 'static,
    {
        Sse {
            stream: stream.boxed(),
            keep_alive: None,
        }
    }

    pub fn keep_alive(mut self, keep_alive: KeepAlive) -> Self {
        self.keep_alive = Some(keep_alive);
        self
    }
}

#[derive(Debug, Default, Clone)]
pub struct Event {
    buffer: BytesMut,
    flags: EventFlags,
}

impl Event {
    pub fn data<T>(mut self, data: T) -> Event
    where
        T: AsRef<str>,
    {
        if self.flags.contains(EventFlags::HAS_DATA) {
            panic!("Called `EventBuilder::data` multiple times");
        }

        for line in memchr_split(b'\n', data.as_ref().as_bytes()) {
            self.field("data", line);
        }

        self.flags.insert(EventFlags::HAS_DATA);

        self
    }

    pub fn comment<T>(mut self, comment: T) -> Event
    where
        T: AsRef<str>,
    {
        self.field("", comment.as_ref());
        self
    }

    pub fn event<T>(mut self, event: T) -> Event
    where
        T: AsRef<str>,
    {
        if self.flags.contains(EventFlags::HAS_EVENT) {
            panic!("Called `EventBuilder::event` multiple times");
        }
        self.flags.insert(EventFlags::HAS_EVENT);

        self.field("event", event.as_ref());

        self
    }

    pub fn retry(mut self, duration: Duration) -> Event {
        if self.flags.contains(EventFlags::HAS_RETRY) {
            panic!("Called `EventBuilder::retry` multiple times");
        }
        self.flags.insert(EventFlags::HAS_RETRY);

        self.buffer.extend_from_slice(b"retry:");

        let secs = duration.as_secs();
        let millis = duration.subsec_millis();

        if secs > 0 {
            self.buffer
                .extend_from_slice(itoa::Buffer::new().format(secs).as_bytes());

            if millis < 10 {
                self.buffer.extend_from_slice(b"00");
            } else if millis < 100 {
                self.buffer.extend_from_slice(b"0");
            }
        }

        self.buffer
            .extend_from_slice(itoa::Buffer::new().format(millis).as_bytes());

        self.buffer.put_u8(b'\n');

        self
    }

    pub fn id<T>(mut self, id: T) -> Event
    where
        T: AsRef<str>,
    {
        if self.flags.contains(EventFlags::HAS_ID) {
            panic!("Called `EventBuilder::id` multiple times");
        }
        self.flags.insert(EventFlags::HAS_ID);

        let id = id.as_ref().as_bytes();
        assert_eq!(
            memchr::memchr(b'\0', id),
            None,
            "Event ID cannot contain null characters",
        );

        self.field("id", id);
        self
    }

    fn field(&mut self, name: &str, value: impl AsRef<[u8]>) {
        let value = value.as_ref();
        assert_eq!(
            memchr::memchr2(b'\r', b'\n', value),
            None,
            "SSE field value cannot contain newlines or carriage returns",
        );
        self.buffer.extend_from_slice(name.as_bytes());
        self.buffer.put_u8(b':');
        self.buffer.put_u8(b' ');
        self.buffer.extend_from_slice(value);
        self.buffer.put_u8(b'\n');
    }

    fn finalize(mut self) -> Bytes {
        self.buffer.put_u8(b'\n');
        self.buffer.freeze()
    }
}

#[derive(Default, Debug, Copy, Clone, PartialEq)]
struct EventFlags(u8);

impl EventFlags {
    const HAS_DATA: Self = Self::from_bits(0b0001);
    const HAS_EVENT: Self = Self::from_bits(0b0010);
    const HAS_RETRY: Self = Self::from_bits(0b0100);
    const HAS_ID: Self = Self::from_bits(0b1000);

    const fn bits(&self) -> u8 {
        self.0
    }

    const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }

    const fn contains(&self, other: Self) -> bool {
        self.bits() & other.bits() == other.bits()
    }

    fn insert(&mut self, other: Self) {
        *self = Self::from_bits(self.bits() | other.bits());
    }
}

fn memchr_split(needle: u8, haystack: &[u8]) -> MemchrSplit<'_> {
    MemchrSplit {
        needle,
        haystack: Some(haystack),
    }
}

struct MemchrSplit<'a> {
    needle: u8,
    haystack: Option<&'a [u8]>,
}

impl<'a> Iterator for MemchrSplit<'a> {
    type Item = &'a [u8];
    fn next(&mut self) -> Option<Self::Item> {
        let haystack = self.haystack?;
        if let Some(pos) = memchr::memchr(self.needle, haystack) {
            let (front, back) = haystack.split_at(pos);
            self.haystack = Some(&back[1..]);
            Some(front)
        } else {
            self.haystack.take()
        }
    }
}

pin_project! {
    struct SseBody {
        event_stream: BoxStream<'static, Event>,
        #[pin]
        keep_alive: Option<KeepAliveStream>,
    }
}

impl http_body::Body for SseBody {
    type Data = Bytes;
    type Error = Infallible;

    fn poll_frame(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        let this = self.project();

        match this.event_stream.as_mut().poll_next(cx) {
            Poll::Pending => {
                if let Some(keep_alive) = this.keep_alive.as_pin_mut() {
                    keep_alive.poll_event(cx).map(|e| Some(Ok(Frame::data(e))))
                } else {
                    Poll::Pending
                }
            }
            Poll::Ready(Some(event)) => {
                if let Some(keep_alive) = this.keep_alive.as_pin_mut() {
                    keep_alive.reset();
                }
                Poll::Ready(Some(Ok(Frame::data(event.finalize()))))
            }
            Poll::Ready(None) => Poll::Ready(None),
        }
    }
}

#[derive(Debug, Clone)]
#[must_use]
pub struct KeepAlive {
    event: Bytes,
    max_interval: Duration,
}

impl Default for KeepAlive {
    fn default() -> Self {
        Self::new()
    }
}

impl KeepAlive {
    pub fn new() -> Self {
        Self {
            event: Bytes::from_static(b":\n\n"),
            max_interval: Duration::from_secs(15),
        }
    }

    pub fn interval(mut self, time: Duration) -> Self {
        self.max_interval = time;
        self
    }

    pub fn text<I>(self, text: I) -> Self
    where
        I: AsRef<str>,
    {
        self.event(Event::default().comment(text))
    }

    pub fn event(mut self, event: Event) -> Self {
        self.event = event.finalize();
        self
    }
}

pin_project! {
    #[derive(Debug)]
    struct KeepAliveStream {
        keep_alive: KeepAlive,
        #[pin]
        alive_timer: Sleep,
    }
}

impl KeepAliveStream {
    fn new(keep_alive: KeepAlive) -> Self {
        Self {
            alive_timer: tokio::time::sleep(keep_alive.max_interval),
            keep_alive,
        }
    }

    fn reset(self: Pin<&mut Self>) {
        let this = self.project();
        this.alive_timer
            .reset(tokio::time::Instant::now() + this.keep_alive.max_interval);
    }

    fn poll_event(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Bytes> {
        let this = self.as_mut().project();

        ready!(this.alive_timer.poll(cx));

        let event = this.keep_alive.event.clone();

        self.reset();

        Poll::Ready(event)
    }
}
