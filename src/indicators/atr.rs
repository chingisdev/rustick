use std::collections::HashSet;
use ndarray::{s, Array1};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use rayon::prelude::*;
use crate::indicators::utils::validate_parameter_within_data_length;
use crate::models::data::{BarField, InputData, OutputData};
use crate::models::groups::{CalculationMethodology, ComplexityLevel, DataInputType, Group, MarketSuitability, MathematicalBasis, OutputFormat, SignalInterpretation, SignalType, SmoothingTechnique, TimeframeFocus, TradingStrategySuitability, UseCase};
use crate::models::indicator::{Indicator, IndicatorError};
use crate::validation::validator::{IParameter, ParamRule, Validator};

#[derive(Serialize, Deserialize)]
pub struct ATRParams {
    #[serde(default = "default_period")]
    pub period: usize,
}

fn default_period() -> usize { 14 }

impl IParameter for ATRParams {}

pub struct ATR {
    groups: HashSet<Group>,
    validator: Validator,
}

fn create_validator() -> Validator {
    Validator::new(
        vec![BarField::HIGH, BarField::LOW, BarField::CLOSE],
        vec![
            ParamRule::Required("period"),
            ParamRule::PositiveInteger("period"),
            ParamRule::Custom(Box::new(|value: &Value, data: &InputData| validate_parameter_within_data_length(value, data, "period", BarField::HIGH))),
        ],
    )
}

fn create_groups() -> HashSet<Group> {
    let mut groups = HashSet::new();
    // Use Case
    groups.insert(Group::UseCase(UseCase::VolatilityMeasurement));
    // Mathematical Basis
    groups.insert(Group::MathematicalBasis(MathematicalBasis::Averaging));
    groups.insert(Group::MathematicalBasis(MathematicalBasis::StatisticalMethods));
    // Data Input Type
    groups.insert(Group::DataInputType(DataInputType::PriceBased));
    // Signal Type
    groups.insert(Group::SignalType(SignalType::Lagging));
    // Output Format
    groups.insert(Group::OutputFormat(OutputFormat::SingleLine));
    groups.insert(Group::OutputFormat(OutputFormat::Absolute));
    // Timeframe Focus
    groups.insert(Group::TimeframeFocus(TimeframeFocus::Short));
    groups.insert(Group::TimeframeFocus(TimeframeFocus::Medium));
    // Complexity Level
    groups.insert(Group::ComplexityLevel(ComplexityLevel::Basic));
    // Market Suitability
    groups.insert(Group::MarketSuitability(MarketSuitability::Trending));
    groups.insert(Group::MarketSuitability(MarketSuitability::RangeBound));
    groups.insert(Group::MarketSuitability(MarketSuitability::Volatile));
    // Trading Strategy Suitability
    groups.insert(Group::TradingStrategySuitability(TradingStrategySuitability::Intraday));
    groups.insert(Group::TradingStrategySuitability(TradingStrategySuitability::Swing));
    // Smoothing Technique
    groups.insert(Group::SmoothingTechnique(SmoothingTechnique::Exponential));
    // Calculation Methodology
    groups.insert(Group::CalculationMethodology(CalculationMethodology::Statistical));
    // Signal Interpretation
    groups.insert(Group::SignalInterpretation(SignalInterpretation::ThresholdLevels));
    groups
}

impl ATR {
    pub fn new() -> Self {
        let groups = create_groups();
        let validator = create_validator();
        Self { groups, validator }
    }
}

impl Indicator for ATR {
    fn short_name(&self) -> &'static str {
        "ATR"
    }

    fn name(&self) -> &'static str {
        "Average True Range"
    }

    fn get_groups(&mut self) -> &HashSet<Group> {
        &self.groups
    }

    fn calculate(&self, data: &InputData, params: Value) -> Result<OutputData, IndicatorError> {
        let params: ATRParams = serde_json::from_value(params)
            .map_err(|e| IndicatorError::InvalidParameters(e.to_string()))?;

        self.validator.validate(data, &params)?;

        let high = data.get_by_bar_field(&BarField::HIGH).unwrap();
        let low = data.get_by_bar_field(&BarField::LOW).unwrap();
        let close = data.get_by_bar_field(&BarField::CLOSE).unwrap();
        let length = high.len();
        let period = params.period;

        // Calculate True Range (TR)
        let tr_vec: Vec<f64> = (1..length)
            .into_par_iter()
            .map(|i| {
                let hl = high[i] - low[i];
                let hpc = (high[i] - close[i - 1]).abs();
                let lpc = (low[i] - close[i - 1]).abs();
                hl.max(hpc).max(lpc)
            })
            .collect();

        // Combine first TR value
        let mut tr = Array1::<f64>::zeros(length);
        tr[0] = high[0] - low[0];
        tr.slice_mut(s![1..]).assign(&Array1::from(tr_vec));

        // Calculate ATR
        let mut atr = Array1::<f64>::from_elem(length, f64::NAN);
        // Initial ATR value as the mean of the first 'period' TR values
        atr[period - 1] = tr.slice(s![0..period]).mean().unwrap();

        // Subsequent ATR values
        let period_f64 = period as f64;
        for i in period..length {
            atr[i] = (atr[i - 1] * (period_f64 - 1.0) + tr[i]) / period_f64;
        }

        Ok(OutputData::SingleSeries(atr))
    }
}

#[cfg(test)]
mod test {
    use ndarray::{array, Array1};
    use serde_json::json;
    use crate::models::data::{InputData, OutputData};
    use super::*;

    #[test]
    fn test_atr_length() {
        // Sample high, low, and close price data
        let high = array![48.70, 48.72, 48.90, 48.87, 48.82];
        let low = array![47.79, 48.14, 48.39, 48.37, 48.24];
        let close = array![48.16, 48.61, 48.75, 48.63, 48.74];

        let input_data = InputData {
            open: None,
            high: Some(high.clone()),
            low: Some(low.clone()),
            close: Some(close.clone()),
            volume: None,
        };

        let indicator = ATR::new();

        let params = json!({ "period": 3 });

        let result = indicator.calculate(&input_data, params).unwrap();

        if let OutputData::SingleSeries(atr_values) = result {
            println!("ATR values: {:?}", atr_values);
            assert_eq!(atr_values.len(), high.len());
        } else {
            panic!("Unexpected output format");
        }
    }

    #[test]
    fn test_atr_expected_nan() {
        // Sample high, low, and close price data
        let high = array![48.70, 48.72, 48.90, 48.87, 48.82];
        let low = array![47.79, 48.14, 48.39, 48.37, 48.24];
        let close = array![48.16, 48.61, 48.75, 48.63, 48.74];

        let input_data = InputData {
            open: None,
            high: Some(high.clone()),
            low: Some(low.clone()),
            close: Some(close.clone()),
            volume: None,
        };

        let indicator = ATR::new();

        let params = json!({ "period": 3 });

        let result = indicator.calculate(&input_data, params).unwrap();

        if let OutputData::SingleSeries(atr_values) = result {
            println!("ATR values: {:?}", atr_values);
            let invalid_length = 3 - 1;
            for i in 0..invalid_length {
                assert!(atr_values[i].is_nan(), "Expected NaN at index {}", i);
            }
        } else {
            panic!("Unexpected output format");
        }
    }

    #[test]
    fn test_atr_expected_value() {
        // Sample high, low, and close price data
        let high = array![48.70, 48.72, 48.90, 48.87, 48.82];
        let low = array![47.79, 48.14, 48.39, 48.37, 48.24];
        let close = array![48.16, 48.61, 48.75, 48.63, 48.74];

        let input_data = InputData {
            open: None,
            high: Some(high.clone()),
            low: Some(low.clone()),
            close: Some(close.clone()),
            volume: None,
        };

        let indicator = ATR::new();

        let params = json!({ "period": 3 });

        let result = indicator.calculate(&input_data, params).unwrap();

        if let OutputData::SingleSeries(atr_values) = result {
            println!("ATR values: {:?}", atr_values);
            let invalid_length = 3 - 1;

            // Remaining values should be valid numbers
            for i in invalid_length..high.len() {
                assert!(
                    !atr_values[i].is_nan(),
                    "Expected valid ATR value at index {}, found NaN",
                    i
                );
            }
        } else {
            panic!("Unexpected output format");
        }
    }

    #[test]
    fn test_atr_expected_first_value() {
        // Sample high, low, and close price data
        let high = array![48.70, 48.72, 48.90, 48.87, 48.82];
        let low = array![47.79, 48.14, 48.39, 48.37, 48.24];
        let close = array![48.16, 48.61, 48.75, 48.63, 48.74];

        let input_data = InputData {
            open: None,
            high: Some(high.clone()),
            low: Some(low.clone()),
            close: Some(close.clone()),
            volume: None,
        };

        let indicator = ATR::new();

        let params = json!({ "period": 3 });

        let result = indicator.calculate(&input_data, params).unwrap();

        if let OutputData::SingleSeries(atr_values) = result {
            println!("ATR values: {:?}", atr_values);

            let mut tr = Array1::<f64>::zeros(high.len());
            tr[0] = high[0] - low[0];
            for i in 1..high.len() {
                let hl = high[i] - low[i];
                let hpc = (high[i] - close[i - 1]).abs();
                let lpc = (low[i] - close[i - 1]).abs();
                tr[i] = hl.max(hpc).max(lpc);
            }

            let expected_first_atr = (tr[0] + tr[1] + tr[2]) / 3.0;
            assert!(
                (atr_values[2] - expected_first_atr).abs() < 1e-6,
                "ATR value at index 2 does not match expected value"
            );
        } else {
            panic!("Unexpected output format");
        }
    }
}
