use std::path::Path;

use trace_core::Language;
use trace_parser::{read_source, DiscoveryOptions, SyntaxParser};

fn main() {
    let path = std::env::args().nth(1).expect("usage: dump <file>");
    let path = Path::new(&path);
    let language = Language::from_extension(
        path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or_default(),
    )
    .expect("unsupported extension");
    let source = read_source(path, DiscoveryOptions::default().max_file_size).unwrap();
    let mut parser = SyntaxParser::new().unwrap();
    let parsed = parser
        .parse(&path.to_string_lossy(), language, &source)
        .unwrap();
    println!("{}", serde_json::to_string_pretty(&parsed).unwrap());
}
