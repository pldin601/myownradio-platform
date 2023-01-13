use actix_web::web::Bytes;
use std::time::Duration;

#[derive(Clone, Debug)]
pub(crate) struct Buffer {
    bytes: Bytes,
    dts: Duration,
}

impl Buffer {
    pub(crate) fn new(bytes: Bytes, dts: Duration) -> Self {
        Buffer { bytes, dts }
    }

    pub(crate) fn bytes(&self) -> &Bytes {
        &self.bytes
    }

    pub(crate) fn dts(&self) -> &Duration {
        &self.dts
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }

    pub(crate) fn into_bytes(self) -> Bytes {
        self.bytes
    }
}
