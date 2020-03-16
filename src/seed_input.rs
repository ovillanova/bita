use futures_util::future;
use futures_util::stream::StreamExt;
use log::*;
use tokio::io::AsyncRead;

use crate::output::Output;
use crate::string_utils::*;
use bitar::archive_reader::ArchiveReader;
use bitar::chunk_index::ChunkIndex;
use bitar::chunker::{Chunker, ChunkerConfig};
use bitar::error::Error;
use bitar::HashSum;

pub struct SeedInput<'a, I> {
    input: I,
    chunker_config: &'a ChunkerConfig,
    num_chunk_buffers: usize,
    stats: SeedStats,
}

#[derive(Default)]
pub struct SeedStats {
    pub chunks_used: usize,
    pub bytes_used: u64,
}

impl<'a, I> SeedInput<'a, I> {
    pub fn new(input: I, chunker_config: &'a ChunkerConfig, num_chunk_buffers: usize) -> Self {
        Self {
            input,
            chunker_config,
            num_chunk_buffers,
            stats: SeedStats::default(),
        }
    }

    pub async fn seed(
        mut self,
        archive: &ArchiveReader,
        chunks_left: &mut ChunkIndex,
        output: &mut Output,
    ) -> Result<SeedStats, Error>
    where
        I: AsyncRead + Unpin,
    {
        let hash_length = archive.chunk_hash_length();
        let seed_chunker = Chunker::new(self.chunker_config, &mut self.input);
        let mut found_chunks = seed_chunker
            .map(|result| {
                tokio::task::spawn(async move {
                    result
                        .map(|(_offset, chunk)| {
                            (HashSum::b2_digest(&chunk, hash_length as usize), chunk)
                        })
                        .map_err(|err| Error::from(("error while chunking seed", err)))
                })
            })
            .buffered(self.num_chunk_buffers)
            .filter_map(|result| {
                // Filter unique chunks to be compressed
                future::ready(match result {
                    Ok(Ok((hash, chunk))) => {
                        if chunks_left.remove(&hash) {
                            Some(Ok((hash, chunk)))
                        } else {
                            None
                        }
                    }
                    Ok(Err(err)) => Some(Err(err)),
                    Err(err) => Some(Err(("error while chunking seed", err).into())),
                })
            });

        while let Some(result) = found_chunks.next().await {
            let (hash, chunk) = result?;
            debug!("Chunk '{}', size {} used", hash, size_to_str(chunk.len()));
            for offset in archive
                .source_index()
                .offsets(&hash)
                .ok_or_else(|| format!("missing chunk ({}) in source!?", hash))?
            {
                self.stats.bytes_used += chunk.len() as u64;
                output
                    .seek_write(offset, &chunk)
                    .await
                    .map_err(|err| ("error writing output", err))?;
            }
            self.stats.chunks_used += 1;
        }

        Ok(self.stats)
    }
}
