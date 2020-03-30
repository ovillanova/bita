#![recursion_limit = "256"]

pub mod archive;
pub mod archive_reader;
pub mod buzhash;
pub mod chunk_index;
pub mod chunk_location_map;
pub mod chunker;
pub mod compression;
pub mod error;
mod hashsum;
mod http_range_request;
mod reader_backend;
pub mod rolling_hash;
pub mod rollsum;
pub use hashsum::*;

pub use reader_backend::ReaderBackend;

pub mod chunk_dictionary {
    include!(concat!(env!("OUT_DIR"), "/chunk_dictionary.rs"));
}
