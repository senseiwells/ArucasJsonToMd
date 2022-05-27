mod doc_parser;

use std::fs;
use crate::doc_parser::DocParser;

fn main() {
    let parser: DocParser = doc_parser::DocParser::new("AllDocs.json");

    fs::write("Classes.md", parser.parse_classes()).expect("Could not write classes");
    fs::write("Extensions.md", parser.parse_extensions()).expect("Could not write extensions")
}
