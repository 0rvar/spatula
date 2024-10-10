use std::path::PathBuf;

use ariadne::{sources, Color, Label, Report, ReportKind};
use spatula::{
    parser::{parse, Instruction, ParseError, Spanned},
    validator,
};

fn main() {
    let path = std::env::args()
        .nth(1)
        .expect("Expected path to source file");
    let filename = PathBuf::from(&path)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.clone());
    let contents = std::fs::read_to_string(path).expect("Failed to read file");
    let program = match parse(&contents) {
        Ok(ast) => ast,
        Err(error) => {
            match error {
                ParseError::FirstStage(errors) => {
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
                }
                ParseError::SecondStage(error_msg, span) => {
                    Report::build(ReportKind::Error, filename.clone(), span.start)
                        .with_message(error_msg.clone())
                        .with_label(
                            Label::new((filename.clone(), span.into_range()))
                                .with_message(error_msg)
                                .with_color(Color::Red),
                        )
                        .finish()
                        .eprint(sources([(filename.clone(), contents.clone())]))
                        .unwrap();
                }
            }
            std::process::exit(1);
        }
    };

    if let Err(e) = validator::validate(&program) {
        Report::build(ReportKind::Error, filename.clone(), e.span.start)
            .with_message(e.message.clone())
            .with_label(
                Label::new((filename.clone(), e.span.into_range()))
                    .with_message(e.message)
                    .with_color(Color::Red),
            )
            .finish()
            .eprint(sources([(filename.clone(), contents.clone())]))
            .unwrap();
        std::process::exit(1);
    }
    println!("{:#?}", program);

    // for recipe in [ast.main].into_iter().chain(ast.auxilary.into_values()) {
    //     for Spanned(instruction, span) in recipe.instructions {
    //         let Instruction::VerbLoop(vl) = instruction else {
    //             continue;
    //         };

    //         Report::build(ReportKind::Advice, filename.clone(), span.start)
    //             .with_message("Look at this loop bruv")
    //             .with_label(
    //                 Label::new((filename.clone(), span.into_range()))
    //                     .with_message("Here is the loop bruv")
    //                     .with_color(Color::Red),
    //             )
    //             .finish()
    //             .eprint(sources([(filename.clone(), contents.clone())]))
    //             .unwrap();
    //     }
    // }

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
