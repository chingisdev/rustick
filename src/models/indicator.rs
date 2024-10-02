use crate::models::groups::Group;
use std::collections::HashSet;
use crate::models::data::{InputData, OutputData};
use serde_json::Value;

#[derive(Debug)]
pub enum IndicatorError {
    InvalidInput(String),
    InvalidParameters(String),
    CalculationError(String),
}

pub trait Indicator {
    fn short_name(&self) -> &'static str;
    fn name(&self) -> &'static str;
    fn get_groups(&mut self) -> &HashSet<Group>;
    fn calculate(&self, data: &InputData, params: Value) -> Result<OutputData, IndicatorError>;
}