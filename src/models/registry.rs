use std::collections::HashMap;
use crate::models::data::OutputData;
use crate::models::groups::Group;
use crate::models::indicator::{Indicator, IndicatorError};

pub struct IndicatorRegistry {
    indicators: HashMap<String, Vec<Box<dyn Indicator>>>,
    groups: HashMap<Group, Vec<Box<dyn Indicator>>>,
}

pub trait AccessorByName {
    fn get_by_names(&self, names: Vec<&str>) -> Option<Vec<&dyn Indicator>>;
    fn calculate_by_names(&self, names: Vec<&str>) -> Result<Vec<OutputData>, IndicatorError>;
}

pub trait AccessorByGroup {
    fn get_by_groups(&self, groups: Vec<Group>) -> Option<Vec<&dyn Indicator>>;
    fn calculate_by_groups(&self, groups: Vec<Group>) -> Result<Vec<OutputData>, IndicatorError>;
}

pub trait Registry {
    fn register_indicator(&mut self, indicator: Box<dyn Indicator>);
}