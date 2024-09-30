use crate::models::groups::Group;
use std::collections::HashSet;
use crate::models::data::{InputData, OutputData};
use serde_json::Value;

#[derive(Debug)]
pub enum IndicatorError {
    #[error("Invalid parameters: {0}")]
    InvalidParameters(String),
    #[error("Calculation error: {0}")]
    CalculationError(String),
}

pub trait Indicator {
    fn name(&self) -> String;
    fn groups(&self) -> HashSet<Group>;
    fn calculate(&self, data: &InputData, params: Value) -> Result<OutputData, IndicatorError>;
}