use blake3;

use crate::syntax::ParsedFile;

use super::SyntaxBuilder;

impl<'a> SyntaxBuilder<'a> {
    pub(crate) fn finish(self) -> ParsedFile {
        ParsedFile {
            path: self.path.to_string(),
            language: self.language,
            source_hash: blake3::hash(self.source.as_bytes()).to_hex().to_string(),
            module_name: self.module_name,
            symbols: self.symbols,
            imports: self.imports,
            calls: self.calls,
            type_refs: self.type_refs,
            heritage: self.heritage,
            locals: self.locals,
            diagnostics: Vec::new(),
            parse_error_count: 0,
        }
    }
}
