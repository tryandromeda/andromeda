// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at https://mozilla.org/MPL/2.0/.

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Write;

use andromeda_core::Rid;
use brotli::DecompressorWriter as BrotliWriter;
use flate2::write::{GzDecoder, ZlibDecoder};

/// Per-rid streaming decoder state. Each variant takes raw input chunks and
/// emits decoded bytes.
pub(crate) enum Decompressor {
    Gzip(Box<GzDecoder<Vec<u8>>>),
    /// `Content-Encoding: deflate` — zlib-wrapped per the spec.
    Deflate(Box<ZlibDecoder<Vec<u8>>>),
    Brotli(Box<BrotliWriter<Vec<u8>>>),
}

impl Decompressor {
    pub(crate) fn from_encoding(encoding: &str) -> Option<Self> {
        match encoding.trim().to_ascii_lowercase().as_str() {
            "gzip" | "x-gzip" => Some(Decompressor::Gzip(Box::new(GzDecoder::new(Vec::new())))),
            "deflate" => Some(Decompressor::Deflate(Box::new(ZlibDecoder::new(Vec::new())))),
            "br" => Some(Decompressor::Brotli(Box::new(BrotliWriter::new(Vec::new(), 8192)))),
            _ => None,
        }
    }

    /// Feed a chunk of compressed input; return the bytes decoded so far.
    pub(crate) fn decode(&mut self, input: &[u8]) -> Result<Vec<u8>, String> {
        match self {
            Decompressor::Gzip(dec) => {
                dec.write_all(input).map_err(|e| e.to_string())?;
                Ok(std::mem::take(dec.get_mut()))
            }
            Decompressor::Deflate(dec) => {
                dec.write_all(input).map_err(|e| e.to_string())?;
                Ok(std::mem::take(dec.get_mut()))
            }
            Decompressor::Brotli(dec) => {
                dec.write_all(input).map_err(|e| e.to_string())?;
                dec.flush().map_err(|e| e.to_string())?;
                Ok(std::mem::take(dec.get_mut()))
            }
        }
    }

    /// Flush trailing bytes when the underlying socket reaches EOF.
    pub(crate) fn finish(self) -> Result<Vec<u8>, String> {
        match self {
            Decompressor::Gzip(dec) => (*dec).finish().map_err(|e| e.to_string()),
            Decompressor::Deflate(dec) => (*dec).finish().map_err(|e| e.to_string()),
            Decompressor::Brotli(mut dec) => {
                dec.flush().map_err(|e| e.to_string())?;
                Ok(std::mem::take(dec.get_mut()))
            }
        }
    }
}

#[derive(Default)]
pub(crate) struct DecompressionResources {
    pub(crate) decoders: RefCell<HashMap<Rid, Decompressor>>,
}

impl DecompressionResources {
    pub(crate) fn new() -> Self {
        Self {
            decoders: RefCell::new(HashMap::new()),
        }
    }

    /// Returns true if any decoder is registered for this rid.
    pub(crate) fn has(&self, rid: Rid) -> bool {
        self.decoders.borrow().contains_key(&rid)
    }

    /// Decode a raw chunk through the active decoder, or return it
    /// untouched if no decoder is registered.
    pub(crate) fn decode_chunk(&self, rid: Rid, input: &[u8]) -> Result<Vec<u8>, String> {
        let mut map = self.decoders.borrow_mut();
        match map.get_mut(&rid) {
            Some(dec) => dec.decode(input),
            None => Ok(input.to_vec()),
        }
    }

    /// Flush trailing bytes for this rid (EOF). Removes the entry; returns
    /// any tail bytes the decoder still owed. Empty Vec if no decoder.
    pub(crate) fn finish_chunk(&self, rid: Rid) -> Result<Vec<u8>, String> {
        let dec_opt = self.decoders.borrow_mut().remove(&rid);
        match dec_opt {
            Some(dec) => dec.finish(),
            None => Ok(Vec::new()),
        }
    }

    pub(crate) fn insert(&self, rid: Rid, dec: Decompressor) {
        self.decoders.borrow_mut().insert(rid, dec);
    }
}
