pub mod http1;
pub mod http2;

#[cfg(test)]
pub mod http1_tests;
#[cfg(test)]
pub mod http2_tests;

pub use http1::{Http1Client, Http1Error, Http1Method, Http1Request, Http1Response, Http1Server};
pub use http2::{HpackCodec, Http2Client, Http2Error, Http2RequestPayload, Http2Response, Http2Server};

