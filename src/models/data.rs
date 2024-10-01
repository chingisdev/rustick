use ndarray::{Array1, Array2};

pub struct InputData {
    pub open: Option<Array1<f64>>,
    pub high: Option<Array1<f64>>,
    pub low: Option<Array1<f64>>,
    pub close: Option<Array1<f64>>,
    pub volume: Option<Array1<f64>>,
}

pub enum OutputData {
    SingleSeries(Array1<f64>),
    MultiSeries(Array2<f64>)
}