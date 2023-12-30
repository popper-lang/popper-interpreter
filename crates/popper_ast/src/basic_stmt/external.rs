use crate::{Span, FunctionSign};

#[cfg_attr(feature = "extra-trait", derive(Debug, PartialEq))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone)]
pub struct External {
    span: Span,
    pub signs: Vec<FunctionSign>
}

impl External {
    pub fn new(span: Span, signs: Vec<FunctionSign>) -> Self {
        Self { span, signs }
    }

    pub fn span(&self) -> Span {
        self.span
    }
}