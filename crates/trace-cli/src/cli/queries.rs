//! Query commands: callers, callees, refs, deps, impact, path.

use std::path::Path;

use anyhow::Result;

pub(crate) fn cmd_callers(
    symbol: &str,
    path: &Path,
    max_depth: usize,
    max_results: usize,
) -> Result<()> {
    let _ = (symbol, path, max_depth, max_results);
    println!("callers not yet implemented");
    Ok(())
}

pub(crate) fn cmd_callees(
    symbol: &str,
    path: &Path,
    max_depth: usize,
    max_results: usize,
) -> Result<()> {
    let _ = (symbol, path, max_depth, max_results);
    println!("callees not yet implemented");
    Ok(())
}

pub(crate) fn cmd_refs(
    symbol: &str,
    path: &Path,
    max_depth: usize,
    max_results: usize,
) -> Result<()> {
    let _ = (symbol, path, max_depth, max_results);
    println!("refs not yet implemented");
    Ok(())
}

pub(crate) fn cmd_deps(
    target: &str,
    path: &Path,
    transitive: bool,
    max_depth: usize,
) -> Result<()> {
    let _ = (target, path, transitive, max_depth);
    println!("deps not yet implemented");
    Ok(())
}

pub(crate) fn cmd_impact(
    target: &str,
    path: &Path,
    max_depth: usize,
    max_results: usize,
) -> Result<()> {
    let _ = (target, path, max_depth, max_results);
    println!("impact not yet implemented");
    Ok(())
}

pub(crate) fn cmd_path(source: &str, target: &str, path: &Path, max_depth: usize) -> Result<()> {
    let _ = (source, target, path, max_depth);
    println!("path not yet implemented");
    Ok(())
}
