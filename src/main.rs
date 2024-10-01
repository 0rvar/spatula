use std::path::PathBuf;

use ariadne::{sources, Color, Label, Report, ReportKind};
use spatula::parser::parse;

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("Expected path to source file");
    let filename = PathBuf::from(&path)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.clone());
    let contents = std::fs::read_to_string(path).expect("Failed to read file");
    let ast = match parse(&contents) {
        Ok(ast) => ast,
        Err(errors) => {
            for e in errors {
                Report::build(ReportKind::Error, filename.clone(), e.span().start)
                    .with_message(e.to_string())
                    .with_label(
                        Label::new((filename.clone(), e.span().into_range()))
                            .with_message(e.reason().to_string())
                            .with_color(Color::Red),
                    )
                    .finish()
                    .eprint(sources([(filename.clone(), contents.clone())]))
                    .unwrap();
            }
            std::process::exit(1);
        }
    };
    println!("{:#?}", ast);
    // if let Err(e) = interpreter::Interpreter::new().run_program(ast) {
    //     Report::build(ReportKind::Error, filename.clone(), e.span().start)
    //         .with_message(e.to_string())
    //         .with_label(
    //             Label::new((filename.clone(), e.span().into_range()))
    //                 .with_message(e.reason().to_string())
    //                 .with_color(Color::Red),
    //         )
    //         .finish()
    //         .print(sources([(filename.clone(), contents.clone())]))
    //         .unwrap();
    //     std::process::exit(1);
    // }
}
