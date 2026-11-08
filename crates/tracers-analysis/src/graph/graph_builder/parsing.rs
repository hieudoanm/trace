//! File parsing: sequential and parallel parsing of discovered source files.

use blake3;

use tracers_core::diagnostic::{Diagnostic, DiagnosticKind};
use tracers_parser::discovery::DiscoveredFile;
use tracers_parser::parser::{read_source, SyntaxParser};
use tracers_parser::ParsedFile;

use crate::cache::SyntaxCache;
use crate::error::AnalysisError;

/// Outcome of parsing one discovered file: either the parsed result or a
/// diagnostic explaining why the file was skipped. Per-file failures never
/// abort the rest of the scan.
pub type FileParseOutcome = Result<ParsedFile, Diagnostic>;

/// Parse discovered files, returning parsed files and per-file diagnostics.
/// `max_file_size` is enforced again at read time so files that grow between
/// discovery and parsing cannot exceed the configured limit.
pub fn parse_files(
    files: &[DiscoveredFile],
    jobs: Option<usize>,
    cache: Option<&SyntaxCache>,
    max_file_size: u64,
) -> Result<(Vec<ParsedFile>, Vec<Diagnostic>), AnalysisError> {
    let outcomes = match jobs {
        Some(1) => parse_files_sequential(files, cache, max_file_size),
        Some(n) => parse_files_parallel(files, cache, n, max_file_size),
        None => parse_files_parallel(files, cache, 0, max_file_size),
    }?;
    let mut parsed = Vec::new();
    let mut diagnostics = Vec::new();
    for outcome in outcomes {
        match outcome {
            Ok(file) => parsed.push(file),
            Err(diagnostic) => diagnostics.push(diagnostic),
        }
    }
    Ok((parsed, diagnostics))
}

fn file_diagnostic(file: &DiscoveredFile, kind: DiagnosticKind, message: String) -> Diagnostic {
    Diagnostic::warning(kind, message)
        .with_file(file.path.clone())
        .with_language(file.language)
}

fn parse_with_cache(
    parser: &mut SyntaxParser,
    file: &DiscoveredFile,
    source: &str,
    source_hash: &str,
    cache: Option<&SyntaxCache>,
) -> FileParseOutcome {
    if let Some(cache) = cache {
        if let Some(cached) = cache.load(&file.path, file.language, source_hash) {
            tracing::trace!(path = %file.path, "syntax cache hit");
            return Ok(cached);
        }
        tracing::trace!(path = %file.path, "syntax cache miss");
    }
    let parsed = parser
        .parse(&file.path, file.language, source)
        .map_err(|error| {
            file_diagnostic(
                file,
                DiagnosticKind::ParseError,
                format!("parse failed: {error}"),
            )
        })?;
    if let Some(cache) = cache {
        cache.store(&parsed);
    }
    Ok(parsed)
}

fn parse_files_parallel(
    files: &[DiscoveredFile],
    cache: Option<&SyntaxCache>,
    jobs: usize,
    max_file_size: u64,
) -> Result<Vec<FileParseOutcome>, AnalysisError> {
    use rayon::prelude::*;

    let cache_ref = cache.cloned();
    let run = || {
        files
            .par_iter()
            // One parser per worker thread instead of one per file.
            .map_init(
                || SyntaxParser::new().map_err(|e| e.to_string()).ok(),
                |parser, file| {
                    parse_one_file(parser.as_mut(), file, cache_ref.as_ref(), max_file_size)
                },
            )
            .collect::<Vec<FileParseOutcome>>()
    };
    let outcomes = if jobs == 0 {
        run()
    } else {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(jobs)
            .build()
            .map_err(|e| AnalysisError::Parser(format!("failed to build thread pool: {e}")))?;
        pool.install(run)
    };
    Ok(outcomes)
}

fn parse_files_sequential(
    files: &[DiscoveredFile],
    cache: Option<&SyntaxCache>,
    max_file_size: u64,
) -> Result<Vec<FileParseOutcome>, AnalysisError> {
    let mut parser =
        SyntaxParser::new().map_err(|error| AnalysisError::Parser(error.to_string()))?;
    Ok(files
        .iter()
        .map(|file| parse_one_file(Some(&mut parser), file, cache, max_file_size))
        .collect())
}

fn parse_one_file(
    parser: Option<&mut SyntaxParser>,
    file: &DiscoveredFile,
    cache: Option<&SyntaxCache>,
    max_file_size: u64,
) -> FileParseOutcome {
    let source = match read_source(&file.absolute, max_file_size) {
        Ok(source) => source,
        Err(error) => {
            return Err(file_diagnostic(
                file,
                DiagnosticKind::UnreadableFile,
                format!("cannot read source: {error}"),
            ));
        }
    };
    let source_hash = blake3::hash(source.as_bytes()).to_hex().to_string();
    let Some(parser) = parser else {
        return Err(file_diagnostic(
            file,
            DiagnosticKind::ParseError,
            "syntax parser unavailable on this worker".to_string(),
        ));
    };
    parse_with_cache(parser, file, &source, &source_hash, cache)
}
