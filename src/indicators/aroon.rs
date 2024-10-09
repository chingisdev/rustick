use std::collections::{HashMap, HashSet};
use ndarray::{s, Array1};
use ndarray_stats::QuantileExt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::indicators::utils::validate_parameter_within_data_length;
use crate::models::data::{BarField, InputData, OutputData};
use crate::models::groups::{CalculationMethodology, ComplexityLevel, DataInputType, Group, MarketSuitability, MathematicalBasis, OutputFormat, SignalInterpretation, SignalType, SmoothingTechnique, TimeframeFocus, TradingStrategySuitability, UseCase};
use crate::models::indicator::{Indicator, IndicatorError};
use crate::validation::validator::{IParameter, ParamRule, Validator};

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
        vec![BarField::HIGH, BarField::LOW],
        vec![
            ParamRule::Required("period"),
            ParamRule::PositiveInteger("period"),
            ParamRule::Custom(Box::new(|value: &Value, data: &InputData| validate_parameter_within_data_length(value, data, "period", BarField::HIGH))),
            ParamRule::Custom(Box::new(|value: &Value, data: &InputData| validate_parameter_within_data_length(value, data, "period", BarField::LOW))),
        ]
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
        let period = params.period;
        let mut up = Array1::<f64>::from_elem(length, f64::NAN);
        let mut down = Array1::<f64>::from_elem(length, f64::NAN);

        // Iterate over the valid indices
        for i in (period - 1)..length {
            let start_index = i + 1 - period;
            let end_index = i + 1;

            // Extract the window slices
            let high_window = high.slice(s![start_index..end_index]);
            let low_window = low.slice(s![start_index..end_index]);

            // Use ndarray-stats methods to find the index of the max and min
            let high_max_index = high_window.argmax().unwrap();
            let low_min_index = low_window.argmin().unwrap();

            // Calculate Aroon Up and Aroon Down
            up[i] = ((high_max_index + 1) as f64 / period as f64) * 100.0;
            down[i] = ((low_min_index + 1) as f64 / period as f64) * 100.0;
        }

        let mut output = HashMap::new();
        output.insert("aroon_up", up);
        output.insert("aroon_down", down);

        Ok(OutputData::MultiSeries(output))
    }
}


#[cfg(test)]
mod test {
    use ndarray::array;
    use serde_json::json;
    use crate::models::data::{InputData, OutputData};
    use super::*;

    #[test]
    fn test_aroon_up_length() {
        // Sample high and low price data
        let high = array![1.0, 2.0, 3.0, 4.0, 3.5, 3.0, 2.5, 2.0, 1.5, 1.0];
        let low = array![0.5, 1.0, 1.5, 2.0, 1.8, 1.5, 1.2, 1.0, 0.8, 0.5];

        let input_data = InputData {
            open: None,
            high: Some(high.clone()),
            low: Some(low.clone()),
            close: None,
            volume: None,
        };

        let indicator = AROON::new();

        let params = json!({ "period": 5 });

        let result = indicator.calculate(&input_data, params).unwrap();

        if let OutputData::MultiSeries(output) = result {
            let aroon_up = output.get("aroon_up").unwrap();

            println!("Aroon Up: {:?}", aroon_up);

            // Assert the length is the same as input
            assert_eq!(aroon_up.len(), high.len());
        } else {
            panic!("Unexpected output format");
        }
    }

    #[test]
    fn test_aroon_down_length() {
        // Sample high and low price data
        let high = array![1.0, 2.0, 3.0, 4.0, 3.5, 3.0, 2.5, 2.0, 1.5, 1.0];
        let low = array![0.5, 1.0, 1.5, 2.0, 1.8, 1.5, 1.2, 1.0, 0.8, 0.5];

        let input_data = InputData {
            open: None,
            high: Some(high.clone()),
            low: Some(low.clone()),
            close: None,
            volume: None,
        };

        let indicator = AROON::new();

        let params = json!({ "period": 5 });

        let result = indicator.calculate(&input_data, params).unwrap();

        if let OutputData::MultiSeries(output) = result {
            let aroon_down = output.get("aroon_down").unwrap();

            println!("Aroon Down: {:?}", aroon_down);

            // Assert the length is the same as input
            assert_eq!(aroon_down.len(), high.len());
        } else {
            panic!("Unexpected output format");
        }
    }

    #[test]
    fn test_aroon_up_expected_nan() {
        // Sample high and low price data
        let high = array![1.0, 2.0, 3.0, 4.0, 3.5, 3.0, 2.5, 2.0, 1.5, 1.0];
        let low = array![0.5, 1.0, 1.5, 2.0, 1.8, 1.5, 1.2, 1.0, 0.8, 0.5];

        let input_data = InputData {
            open: None,
            high: Some(high.clone()),
            low: Some(low.clone()),
            close: None,
            volume: None,
        };

        let indicator = AROON::new();

        let params = json!({ "period": 5 });

        let result = indicator.calculate(&input_data, params).unwrap();

        if let OutputData::MultiSeries(output) = result {
            let aroon_up = output.get("aroon_up").unwrap();

            // The first (period - 1) values should be NaN
            let invalid_length = 5 - 1;
            for i in 0..invalid_length {
                assert!(aroon_up[i].is_nan(), "Expected NaN at index {}", i);
            }
        } else {
            panic!("Unexpected output format");
        }
    }

    #[test]
    fn test_aroon_down_expected_nan() {
        // Sample high and low price data
        let high = array![1.0, 2.0, 3.0, 4.0, 3.5, 3.0, 2.5, 2.0, 1.5, 1.0];
        let low = array![0.5, 1.0, 1.5, 2.0, 1.8, 1.5, 1.2, 1.0, 0.8, 0.5];

        let input_data = InputData {
            open: None,
            high: Some(high.clone()),
            low: Some(low.clone()),
            close: None,
            volume: None,
        };

        let indicator = AROON::new();

        let params = json!({ "period": 5 });

        let result = indicator.calculate(&input_data, params).unwrap();

        if let OutputData::MultiSeries(output) = result {
            let aroon_down = output.get("aroon_down").unwrap();

            // The first (period - 1) values should be NaN
            let invalid_length = 5 - 1;
            for i in 0..invalid_length {
                assert!(aroon_down[i].is_nan(), "Expected NaN at index {}", i);
            }

        } else {
            panic!("Unexpected output format");
        }
    }

    #[test]
    fn test_aroon_up_has_value() {
        // Sample high and low price data
        let high = array![1.0, 2.0, 3.0, 4.0, 3.5, 3.0, 2.5, 2.0, 1.5, 1.0];
        let low = array![0.5, 1.0, 1.5, 2.0, 1.8, 1.5, 1.2, 1.0, 0.8, 0.5];

        let input_data = InputData {
            open: None,
            high: Some(high.clone()),
            low: Some(low.clone()),
            close: None,
            volume: None,
        };

        let indicator = AROON::new();

        let params = json!({ "period": 5 });

        let result = indicator.calculate(&input_data, params).unwrap();

        if let OutputData::MultiSeries(output) = result {
            let aroon_up = output.get("aroon_up").unwrap();

            // The first (period - 1) values should be NaN
            let invalid_length = 5 - 1;

            // Remaining values should be valid numbers between 0 and 100
            for i in invalid_length..high.len() {
                assert!(
                    !aroon_up[i].is_nan(),
                    "Expected valid Aroon Up value at index {}, found NaN",
                    i
                );
            }
        } else {
            panic!("Unexpected output format");
        }
    }

    #[test]
    fn test_aroon_down_has_value() {
        // Sample high and low price data
        let high = array![1.0, 2.0, 3.0, 4.0, 3.5, 3.0, 2.5, 2.0, 1.5, 1.0];
        let low = array![0.5, 1.0, 1.5, 2.0, 1.8, 1.5, 1.2, 1.0, 0.8, 0.5];

        let input_data = InputData {
            open: None,
            high: Some(high.clone()),
            low: Some(low.clone()),
            close: None,
            volume: None,
        };

        let indicator = AROON::new();

        let params = json!({ "period": 5 });

        let result = indicator.calculate(&input_data, params).unwrap();

        if let OutputData::MultiSeries(output) = result {
            let aroon_down = output.get("aroon_down").unwrap();

            // The first (period - 1) values should be NaN
            let invalid_length = 5 - 1;

            // Remaining values should be valid numbers between 0 and 100
            for i in invalid_length..high.len() {
                assert!(
                    !aroon_down[i].is_nan(),
                    "Expected valid Aroon Down value at index {}, found NaN",
                    i
                );
            }
        } else {
            panic!("Unexpected output format");
        }
    }

    #[test]
    fn test_aroon_up_within_bounds() {
        // Sample high and low price data
        let high = array![1.0, 2.0, 3.0, 4.0, 3.5, 3.0, 2.5, 2.0, 1.5, 1.0];
        let low = array![0.5, 1.0, 1.5, 2.0, 1.8, 1.5, 1.2, 1.0, 0.8, 0.5];

        let input_data = InputData {
            open: None,
            high: Some(high.clone()),
            low: Some(low.clone()),
            close: None,
            volume: None,
        };

        let indicator = AROON::new();

        let params = json!({ "period": 5 });

        let result = indicator.calculate(&input_data, params).unwrap();

        if let OutputData::MultiSeries(output) = result {
            let aroon_up = output.get("aroon_up").unwrap();

            // The first (period - 1) values should be NaN
            let invalid_length = 5 - 1;

            // Remaining values should be valid numbers between 0 and 100
            for i in invalid_length..high.len() {
                assert!(
                    (0.0..=100.0).contains(&aroon_up[i]),
                    "Aroon Up value at index {} is out of bounds: {}",
                    i,
                    aroon_up[i]
                );
            }
        } else {
            panic!("Unexpected output format");
        }
    }

    #[test]
    fn test_aroon_down_within_bounds() {
        // Sample high and low price data
        let high = array![1.0, 2.0, 3.0, 4.0, 3.5, 3.0, 2.5, 2.0, 1.5, 1.0];
        let low = array![0.5, 1.0, 1.5, 2.0, 1.8, 1.5, 1.2, 1.0, 0.8, 0.5];

        let input_data = InputData {
            open: None,
            high: Some(high.clone()),
            low: Some(low.clone()),
            close: None,
            volume: None,
        };

        let indicator = AROON::new();

        let params = json!({ "period": 5 });

        let result = indicator.calculate(&input_data, params).unwrap();

        if let OutputData::MultiSeries(output) = result {
            let aroon_down = output.get("aroon_down").unwrap();

            // The first (period - 1) values should be NaN
            let invalid_length = 5 - 1;

            // Remaining values should be valid numbers between 0 and 100
            for i in invalid_length..high.len() {
                assert!(
                    (0.0..=100.0).contains(&aroon_down[i]),
                    "Aroon Down value at index {} is out of bounds: {}",
                    i,
                    aroon_down[i]
                );
            }
        } else {
            panic!("Unexpected output format");
        }
    }
}