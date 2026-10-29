//! File parsing: sequential and parallel parsing of discovered source files.

use blake3;

use trace_parser::discovery::DiscoveredFile;
use trace_parser::parser::SyntaxParser;
use trace_parser::ParsedFile;

use crate::cache::SyntaxCache;
use crate::error::AnalysisError;

pub(crate) fn parse_files(
    files: &[DiscoveredFile],
    parallel: bool,
    cache: Option<&SyntaxCache>,
) -> Result<Vec<ParsedFile>, AnalysisError> {
    if parallel {
        parse_files_parallel(files, cache)
    } else {
        parse_files_sequential(files, cache)
    }
}

fn parse_files_parallel(
    files: &[DiscoveredFile],
    cache: Option<&SyntaxCache>,
) -> Result<Vec<ParsedFile>, AnalysisError> {
    use rayon::prelude::*;

    let cache_ref = cache.cloned();
    files
        .par_iter()
        .map(|file| parse_one_file(file, &cache_ref))
        .collect()
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
                std::fs::read_to_string(&file.absolute).map_err(|error| AnalysisError::Read {
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
    file: &DiscoveredFile,
    cache: &Option<SyntaxCache>,
) -> Result<ParsedFile, AnalysisError> {
    let source = std::fs::read_to_string(&file.absolute).map_err(|error| AnalysisError::Read {
        path: file.absolute.display().to_string(),
        message: error.to_string(),
    })?;
    let source_hash = blake3::hash(source.as_bytes()).to_hex().to_string();
    let parsed = if let Some(cache) = cache {
        if let Some(cached) = cache.load(&file.path, file.language, &source_hash) {
            cached
        } else {
            let mut parser =
                SyntaxParser::new().map_err(|error| AnalysisError::Parser(error.to_string()))?;
            let parsed = parser
                .parse(&file.path, file.language, &source)
                .map_err(|error| AnalysisError::Parser(error.to_string()))?;
            cache.store(&parsed);
            parsed
        }
    } else {
        let mut parser =
            SyntaxParser::new().map_err(|error| AnalysisError::Parser(error.to_string()))?;
        parser
            .parse(&file.path, file.language, &source)
            .map_err(|error| AnalysisError::Parser(error.to_string()))?
    };
    Ok(parsed)
}
