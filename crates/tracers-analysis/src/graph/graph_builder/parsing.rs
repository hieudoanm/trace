//! File parsing: sequential and parallel parsing of discovered source files.

use blake3;

use tracers_parser::discovery::DiscoveredFile;
use tracers_parser::parser::{read_source, SyntaxParser};
use tracers_parser::ParsedFile;

use crate::cache::SyntaxCache;
use crate::error::AnalysisError;

pub(crate) fn parse_files(
    files: &[DiscoveredFile],
    jobs: Option<usize>,
    cache: Option<&SyntaxCache>,
) -> Result<Vec<ParsedFile>, AnalysisError> {
    match jobs {
        Some(1) => parse_files_sequential(files, cache),
        Some(n) => parse_files_parallel(files, cache, n),
        None => parse_files_parallel(files, cache, 0),
    }
}

fn parse_files_parallel(
    files: &[DiscoveredFile],
    cache: Option<&SyntaxCache>,
    jobs: usize,
) -> Result<Vec<ParsedFile>, AnalysisError> {
    use rayon::prelude::*;

    let cache_ref = cache.cloned();
    let run = || {
        files
            .par_iter()
            // One parser per worker thread instead of one per file.
            .map_init(
                || SyntaxParser::new().map_err(|e| e.to_string()).ok(),
                |parser, file| parse_one_file(parser.as_mut(), file, &cache_ref),
            )
            .collect()
    };
    if jobs == 0 {
        run()
    } else {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(jobs)
            .build()
            .map_err(|e| AnalysisError::Parser(format!("failed to build thread pool: {e}")))?;
        pool.install(run)
    }
}

fn parse_files_sequential(
    files: &[DiscoveredFile],
    cache: Option<&SyntaxCache>,
) -> Result<Vec<ParsedFile>, AnalysisError> {
    let mut parser =
        SyntaxParser::new().map_err(|error| AnalysisError::Parser(error.to_string()))?;
    files
        .iter()
        .map(|file| {
            let source =
                read_source(&file.absolute, u64::MAX).map_err(|error| AnalysisError::Read {
                    path: file.absolute.display().to_string(),
                    message: error.to_string(),
                })?;
            let source_hash = blake3::hash(source.as_bytes()).to_hex().to_string();
            let parsed = if let Some(cache) = cache {
                if let Some(cached) = cache.load(&file.path, file.language, &source_hash) {
                    cached
                } else {
                    let parsed = parser
                        .parse(&file.path, file.language, &source)
                        .map_err(|error| AnalysisError::Parser(error.to_string()))?;
                    cache.store(&parsed);
                    parsed
                }
            } else {
                parser
                    .parse(&file.path, file.language, &source)
                    .map_err(|error| AnalysisError::Parser(error.to_string()))?
            };
            Ok(parsed)
        })
        .collect()
}

fn parse_one_file(
    parser: Option<&mut SyntaxParser>,
    file: &DiscoveredFile,
    cache: &Option<SyntaxCache>,
) -> Result<ParsedFile, AnalysisError> {
    let source = read_source(&file.absolute, u64::MAX).map_err(|error| AnalysisError::Read {
        path: file.absolute.display().to_string(),
        message: error.to_string(),
    })?;
    let source_hash = blake3::hash(source.as_bytes()).to_hex().to_string();
    let parsed = if let Some(cache) = cache {
        if let Some(cached) = cache.load(&file.path, file.language, &source_hash) {
            tracing::trace!(path = %file.path, "syntax cache hit");
            cached
        } else {
            tracing::trace!(path = %file.path, "syntax cache miss");
            let parser = parser.ok_or_else(|| {
                AnalysisError::Parser("syntax parser unavailable on this worker".to_string())
            })?;
            let parsed = parser
                .parse(&file.path, file.language, &source)
                .map_err(|error| AnalysisError::Parser(error.to_string()))?;
            cache.store(&parsed);
            parsed
        }
    } else {
        let parser = parser.ok_or_else(|| {
            AnalysisError::Parser("syntax parser unavailable on this worker".to_string())
        })?;
        parser
            .parse(&file.path, file.language, &source)
            .map_err(|error| AnalysisError::Parser(error.to_string()))?
    };
    Ok(parsed)
}
