pub struct InputData {
    pub open: Option<Vec<f64>>,
    pub high: Option<Vec<f64>>,
    pub low: Option<Vec<f64>>,
    pub close: Option<Vec<f64>>,
    pub volume: Option<Vec<f64>>,
}

pub enum OutputData {
    SingleSeries(Vec<f64>),
    MultiSeries(Vec<Vec<f64>>)
}