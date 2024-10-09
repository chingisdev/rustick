use std::collections::{HashMap, HashSet};
use ndarray::{s, Array1};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::models::data::{BarField, InputData, OutputData};
use crate::models::groups::{CalculationMethodology, ComplexityLevel, DataInputType, Group, MarketSuitability, MathematicalBasis, OutputFormat, SignalInterpretation, SignalType, SmoothingTechnique, TimeframeFocus, TradingStrategySuitability, UseCase};
use crate::models::indicator::{Indicator, IndicatorError};
use crate::validation::validator::{IParameter, Validator};

#[derive(Serialize, Deserialize)]
pub struct AROONParams {
    #[serde(default = "default_period")]
    period: usize,
}

fn default_period() -> usize { 14 }

impl IParameter for AROONParams {}

pub struct AROON {
    groups: HashSet<Group>,
    validator: Validator,
}

fn create_groups() -> HashSet<Group> {
    let mut groups = HashSet::new();
    // Use Case
    groups.insert(Group::UseCase(UseCase::TrendIdentification));
    groups.insert(Group::UseCase(UseCase::MomentumDetection));
    // Mathematical Basis
    groups.insert(Group::MathematicalBasis(MathematicalBasis::Oscillation));
    groups.insert(Group::MathematicalBasis(MathematicalBasis::StatisticalMethods));
    // Data Input Type
    groups.insert(Group::DataInputType(DataInputType::PriceBased));
    // Signal Type
    groups.insert(Group::SignalType(SignalType::Leading));
    // Output Format
    groups.insert(Group::OutputFormat(OutputFormat::MultiLine));
    groups.insert(Group::OutputFormat(OutputFormat::Percentage));
    // Timeframe Focus
    groups.insert(Group::TimeframeFocus(TimeframeFocus::Short));
    groups.insert(Group::TimeframeFocus(TimeframeFocus::Medium));
    // Complexity Level
    groups.insert(Group::ComplexityLevel(ComplexityLevel::Basic));
    // Market Suitability
    groups.insert(Group::MarketSuitability(MarketSuitability::Trending));
    // Trading Strategy Suitability
    groups.insert(Group::TradingStrategySuitability(TradingStrategySuitability::Intraday));
    groups.insert(Group::TradingStrategySuitability(TradingStrategySuitability::Swing));
    // Smoothing Technique
    groups.insert(Group::SmoothingTechnique(SmoothingTechnique::Raw));
    // Calculation Methodology
    groups.insert(Group::CalculationMethodology(CalculationMethodology::Statistical));
    // Signal Interpretation
    groups.insert(Group::SignalInterpretation(SignalInterpretation::Crossovers));
    groups.insert(Group::SignalInterpretation(SignalInterpretation::ThresholdLevels));
    groups
}

fn create_validator() -> Validator {
    Validator::new(
        vec![],
        vec![]
    )
}

impl AROON {
    pub fn new() -> Self {
        let groups = create_groups();
        let validator = create_validator();
        Self { groups, validator }
    }
}

impl Indicator for AROON {
    fn short_name(&self) -> &'static str {
        "AROON"
    }

    fn name(&self) -> &'static str {
        "Aroon"
    }

    fn get_groups(&mut self) -> &HashSet<Group> {
        &self.groups
    }

    fn calculate(&self, data: &InputData, params: Value) -> Result<OutputData, IndicatorError> {
        let params: AROONParams = serde_json::from_value(params)
            .map_err(|e| IndicatorError::InvalidParameters(e.to_string()))?;

        self.validator.validate(data, &params)?;

        let high = data.get_by_bar_field(&BarField::HIGH).unwrap();
        let low = data.get_by_bar_field(&BarField::LOW).unwrap();
        let length = high.len();

        let mut up = Array1::<f64>::from_elem(length, f64::NAN);
        let mut down = Array1::<f64>::from_elem(length, f64::NAN);

        for i in (params.period - 1)..length {
            let start_index = i + 1 - params.period;
            let end_index = i + 1;
            let high_slice = &high.slice(s![start_index..end_index]);
            let low_slice = &low.slice(s![start_index..end_index]);

            let high_max_index = high_slice.argmax().unwrap();
            let low_min_index = low_slice.argmin().unwrap();

            up[i] = ((high_max_index + 1) as f64 / params.period as f64) * 100.0;
            down[i] = ((low_min_index + 1) as f64 / params.period as f64) * 100.0;

        }

        let mut output = HashMap::new();
        output.insert("aroon_up", up);
        output.insert("aroon_down", down);

        Ok(OutputData::MultiSeries(output))
    }
}