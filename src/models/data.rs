use std::collections::HashMap;
use ndarray::{Array1, Array2};

pub struct InputData {
    pub open: Option<Array1<f64>>,
    pub high: Option<Array1<f64>>,
    pub low: Option<Array1<f64>>,
    pub close: Option<Array1<f64>>,
    pub volume: Option<Array1<f64>>,
}

impl InputData {
    pub fn get_by_bar_field(&self, bar_field: &BarField) -> Option<&Array1<f64>> {
        match bar_field {
            BarField::OPEN => self.open.as_ref(),
            BarField::HIGH => self.high.as_ref(),
            BarField::LOW => self.low.as_ref(),
            BarField::CLOSE => self.close.as_ref(),
            BarField::VOLUME => self.volume.as_ref(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum OutputData {
    SingleSeries(Array1<f64>),
    MultiSeries(HashMap<&'static str, Array1<f64>>)
}

pub enum BarField {
    OPEN,
    HIGH,
    LOW,
    CLOSE,
    VOLUME,
}

impl BarField {
    pub fn to_str(&self) -> &str {
        match self {
            BarField::OPEN => "OPEN",
            BarField::HIGH => "HIGH",
            BarField::LOW => "LOW",
            BarField::CLOSE => "CLOSE",
            BarField::VOLUME => "VOLUME",
        }
    }
}