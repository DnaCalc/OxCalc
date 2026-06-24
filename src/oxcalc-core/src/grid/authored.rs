//! Authored grid cell state shared by both grid engines: literal values and
//! formula cells carrying their source text, R1C1-relative normal-form key, and
//! source channel.

use oxfml_core::source::FormulaChannelKind;
use oxfunc_core::value::CalcValue;

#[derive(Debug, Clone, PartialEq)]
pub enum GridAuthoredCell {
    Literal(CalcValue),
    Formula(GridFormulaCell),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridFormulaCell {
    pub source_text: String,
    pub normal_form_key: String,
    pub source_channel: FormulaChannelKind,
}

impl GridFormulaCell {
    #[must_use]
    pub fn new(source_text: impl Into<String>, normal_form_key: impl Into<String>) -> Self {
        Self {
            source_text: source_text.into(),
            normal_form_key: normal_form_key.into(),
            source_channel: FormulaChannelKind::WorksheetA1,
        }
    }

    #[must_use]
    pub const fn with_source_channel(mut self, source_channel: FormulaChannelKind) -> Self {
        self.source_channel = source_channel;
        self
    }
}
