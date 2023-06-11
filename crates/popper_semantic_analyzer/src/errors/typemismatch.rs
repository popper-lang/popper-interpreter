use ast::Span;
use thiserror::Error;
use ariadne::{Color, ColorGenerator, Fmt, Label, Report, ReportKind, Source};
use popper_common::error::{ColorConfig, Error as PopperError};
use popper_common::error::source_to_string;



#[derive(Error, Debug)]
#[error("type mismatch")]
pub struct TypeMismatch {
    pub expected: (Span, String),
    pub found: (Span, String),

}

impl TypeMismatch {
    pub fn new(expected: (Span, String), found: (Span, String)) -> Self {
        Self { expected, found}
    }
}

impl PopperError for TypeMismatch {
    fn report(&self,
              color: ColorConfig,
              source: &Source,
              file: &str)  {
        let type_color = color.get("type").expect("type color not found");

        let mut report = Report::build(ReportKind::Error,
                                       file,
                                       self.expected.0.find_line(
                                           source_to_string(source).as_str()
                                       )
        )
            .with_code(21)
            .with_message(format!("Incompatible types"))
            .with_label(
                Label::new((file, self.expected.0.into()))
                    .with_message(
                        format!("expected type `{}`",
                                self.expected.1.clone().fg(
                                    type_color.clone()
                                )
                        )
                    )
            )
            .with_label(
                Label::new((file, self.found.0.into()))
                    .with_message(
                        format!("found type `{}`",
                                self.found.1.clone().fg(
                                    type_color.clone()
                                )
                        )
                    )
            );


        report.finish().print((file, Source::from(
            source_to_string(source).as_str()
        ))).unwrap();
    }
}