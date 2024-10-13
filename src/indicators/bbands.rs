use std::collections::{HashMap, HashSet};
use ndarray::Array1;
use ndarray::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::indicators::utils::cumulative_sum;
use crate::models::data::{BarField, InputData, OutputData};
use crate::models::groups::{CalculationMethodology, ComplexityLevel, DataInputType, Group, MarketSuitability, MathematicalBasis, OutputFormat, SignalInterpretation, SignalType, SmoothingTechnique, TimeframeFocus, TradingStrategySuitability, UseCase};
use crate::models::indicator::{Indicator, IndicatorError};
use crate::validation::validator::{IParameter, ParamRule, Validator};

pub struct BBands {
    groups: HashSet<Group>,
    validator: Validator,
}

#[derive(Deserialize, Serialize)]
pub struct BBandsParams {
    #[serde(default = "default_period")]
    pub period: usize,
    #[serde(default = "default_std_dev_multiplier")]
    pub std_dev_multiplier: f64,
}

fn default_period() -> usize { 20 }
fn default_std_dev_multiplier() -> f64 { 2.0 }

impl IParameter for BBandsParams {}

fn create_validator() -> Validator {
    Validator::new(
        vec![BarField::CLOSE],
        vec![
            ParamRule::Required("period"),
            ParamRule::Required("std_dev_multiplier"),
            ParamRule::PositiveInteger("period"),
            ParamRule::PositiveNumber("std_dev_multiplier"),
        ],
    )
}

fn create_groups() -> HashSet<Group> {
    let mut groups = HashSet::new();
    groups.insert(Group::UseCase(UseCase::VolatilityMeasurement));
    groups.insert(Group::UseCase(UseCase::TrendIdentification));
    groups.insert(Group::MathematicalBasis(MathematicalBasis::Averaging));
    groups.insert(Group::MathematicalBasis(MathematicalBasis::StatisticalMethods));
    groups.insert(Group::DataInputType(DataInputType::PriceBased));
    groups.insert(Group::SignalType(SignalType::Lagging));
    groups.insert(Group::OutputFormat(OutputFormat::Band));
    groups.insert(Group::OutputFormat(OutputFormat::Absolute));
    groups.insert(Group::TimeframeFocus(TimeframeFocus::Short));
    groups.insert(Group::TimeframeFocus(TimeframeFocus::Medium));
    groups.insert(Group::ComplexityLevel(ComplexityLevel::Intermediate));
    groups.insert(Group::MarketSuitability(MarketSuitability::Trending));
    groups.insert(Group::MarketSuitability(MarketSuitability::RangeBound));
    groups.insert(Group::MarketSuitability(MarketSuitability::Volatile));
    groups.insert(Group::TradingStrategySuitability(TradingStrategySuitability::Intraday));
    groups.insert(Group::TradingStrategySuitability(TradingStrategySuitability::Swing));
    groups.insert(Group::SmoothingTechnique(SmoothingTechnique::SimpleAverage));
    groups.insert(Group::CalculationMethodology(CalculationMethodology::Statistical));
    groups.insert(Group::SignalInterpretation(SignalInterpretation::ThresholdLevels));
    groups.insert(Group::SignalInterpretation(SignalInterpretation::Patterns));
    groups
}

impl BBands {
    pub fn new() -> Self {
        let groups = create_groups();
        let validator = create_validator();
        Self { groups, validator }
    }
}

impl Indicator for BBands {
    fn short_name(&self) -> &'static str {
        "BBANDS"
    }

    fn name(&self) -> &'static str {
        "Bollinger Bands"
    }

    fn get_groups(&mut self) -> &HashSet<Group> {
        &self.groups
    }

    fn calculate(&self, data: &InputData, params: Value) -> Result<OutputData, IndicatorError> {
        let params: BBandsParams = serde_json::from_value(params)
            .map_err(|e| IndicatorError::InvalidParameters(e.to_string()))?;

        self.validator.validate(data, &params)?;

        let close = data.get_by_bar_field(&BarField::CLOSE).unwrap();
        let period = params.period;
        let std_dev_multiplier = params.std_dev_multiplier;
        let length = close.len();

        // Calculate moving average (MA) using vectorized operations
        let mut ma = Array1::<f64>::from_elem(length, f64::NAN);

        // Calculate standard deviation (SD)
        let mut sd = Array1::<f64>::from_elem(length, f64::NAN);

        // Precompute cumulative sums for mean and variance calculations
        let cumsum = cumulative_sum(close);
        let cumsum_sq = cumulative_sum(&(close * close));

        for i in (period - 1)..length {
            let start = i + 1 - period;
            let sum = if start == 0 {
                cumsum[i]
            } else {
                cumsum[i] - cumsum[start - 1]
            };
            let sum_sq = if start == 0 {
                cumsum_sq[i]
            } else {
                cumsum_sq[i] - cumsum_sq[start - 1]
            };

            let mean = sum / period as f64;
            ma[i] = mean;

            let variance = (sum_sq - 2.0 * mean * sum + mean * mean * period as f64) / period as f64;
            let std_dev = variance.sqrt();
            sd[i] = std_dev;
        }

        // Calculate upper and lower bands
        let upper_band = &ma + &(&sd * std_dev_multiplier);
        let lower_band = &ma - &(&sd * std_dev_multiplier);

        // Prepare output data
        let mut output = HashMap::new();
        output.insert("middle_band", ma);
        output.insert("upper_band", upper_band);
        output.insert("lower_band", lower_band);

        Ok(OutputData::MultiSeries(output))
    }
}


#[cfg(test)]
mod test {
    use serde_json::json;
    use super::*;

    #[test]
    fn test_bollinger_bands_middle_length() {
        // Sample close price data
        let close = array![
        22.27, 22.19, 22.08, 22.17, 22.18,
        22.13, 22.23, 22.43, 22.24, 22.29,
        22.15, 22.39, 22.38, 22.61, 23.36,
        24.05, 23.75, 23.83, 23.95, 23.63,
        23.82, 23.87, 23.65, 23.19, 23.10,
        23.33, 22.68, 23.10, 22.40, 22.17
    ];

        let input_data = InputData {
            open: None,
            high: None,
            low: None,
            close: Some(close.clone()),
            volume: None,
        };

        let indicator = BBands::new();

        let params = json!({ "period": 20, "std_dev_multiplier": 2.0 });

        let result = indicator.calculate(&input_data, params).unwrap();

        if let OutputData::MultiSeries(output) = result {
            let middle_band = output.get("middle_band").unwrap();

            println!("Middle Band: {:?}", middle_band);

            assert_eq!(middle_band.len(), close.len());
        } else {
            panic!("Unexpected output format");
        }
    }

    #[test]
    fn test_bollinger_bands_middle_expected_nan() {
        // Sample close price data
        let close = array![
        22.27, 22.19, 22.08, 22.17, 22.18,
        22.13, 22.23, 22.43, 22.24, 22.29,
        22.15, 22.39, 22.38, 22.61, 23.36,
        24.05, 23.75, 23.83, 23.95, 23.63,
        23.82, 23.87, 23.65, 23.19, 23.10,
        23.33, 22.68, 23.10, 22.40, 22.17
    ];

        let input_data = InputData {
            open: None,
            high: None,
            low: None,
            close: Some(close.clone()),
            volume: None,
        };

        let indicator = BBands::new();

        let params = json!({ "period": 20, "std_dev_multiplier": 2.0 });

        let result = indicator.calculate(&input_data, params).unwrap();

        if let OutputData::MultiSeries(output) = result {
            let middle_band = output.get("middle_band").unwrap();

            println!("Middle Band: {:?}", middle_band);

            let invalid_length = 20 - 1;
            for i in 0..invalid_length {
                assert!(middle_band[i].is_nan(), "Expected NaN at index {}", i);
            }
        } else {
            panic!("Unexpected output format");
        }
    }

    #[test]
    fn test_bollinger_bands_middle_expected_value() {
        // Sample close price data
        let close = array![
        22.27, 22.19, 22.08, 22.17, 22.18,
        22.13, 22.23, 22.43, 22.24, 22.29,
        22.15, 22.39, 22.38, 22.61, 23.36,
        24.05, 23.75, 23.83, 23.95, 23.63,
        23.82, 23.87, 23.65, 23.19, 23.10,
        23.33, 22.68, 23.10, 22.40, 22.17
    ];

        let input_data = InputData {
            open: None,
            high: None,
            low: None,
            close: Some(close.clone()),
            volume: None,
        };

        let indicator = BBands::new();

        let params = json!({ "period": 20, "std_dev_multiplier": 2.0 });

        let result = indicator.calculate(&input_data, params).unwrap();

        if let OutputData::MultiSeries(output) = result {
            let middle_band = output.get("middle_band").unwrap();

            println!("Middle Band: {:?}", middle_band);

            let invalid_length = 20 - 1;

            for i in invalid_length..close.len() {
                assert!(
                    !middle_band[i].is_nan(),
                    "Expected valid middle band value at index {}, found NaN",
                    i
                );
            }
        } else {
            panic!("Unexpected output format");
        }
    }

    #[test]
    fn test_bollinger_bands_upper_length() {
        let close = array![
        22.27, 22.19, 22.08, 22.17, 22.18,
        22.13, 22.23, 22.43, 22.24, 22.29,
        22.15, 22.39, 22.38, 22.61, 23.36,
        24.05, 23.75, 23.83, 23.95, 23.63,
        23.82, 23.87, 23.65, 23.19, 23.10,
        23.33, 22.68, 23.10, 22.40, 22.17
    ];

        let input_data = InputData {
            open: None,
            high: None,
            low: None,
            close: Some(close.clone()),
            volume: None,
        };

        let indicator = BBands::new();

        let params = json!({ "period": 20, "std_dev_multiplier": 2.0 });

        let result = indicator.calculate(&input_data, params).unwrap();

        if let OutputData::MultiSeries(output) = result {
            let upper_band = output.get("upper_band").unwrap();

            println!("Upper Band: {:?}", upper_band);

            assert_eq!(upper_band.len(), close.len());
        } else {
            panic!("Unexpected output format");
        }
    }

    #[test]
    fn test_bollinger_bands_upper_expected_nan() {
        let close = array![
        22.27, 22.19, 22.08, 22.17, 22.18,
        22.13, 22.23, 22.43, 22.24, 22.29,
        22.15, 22.39, 22.38, 22.61, 23.36,
        24.05, 23.75, 23.83, 23.95, 23.63,
        23.82, 23.87, 23.65, 23.19, 23.10,
        23.33, 22.68, 23.10, 22.40, 22.17
    ];

        let input_data = InputData {
            open: None,
            high: None,
            low: None,
            close: Some(close.clone()),
            volume: None,
        };

        let indicator = BBands::new();

        let params = json!({ "period": 20, "std_dev_multiplier": 2.0 });

        let result = indicator.calculate(&input_data, params).unwrap();

        if let OutputData::MultiSeries(output) = result {
            let upper_band = output.get("upper_band").unwrap();

            println!("Upper Band: {:?}", upper_band);

            let invalid_length = 20 - 1;
            for i in 0..invalid_length {
                assert!(upper_band[i].is_nan(), "Expected NaN at index {}", i);
            }
        } else {
            panic!("Unexpected output format");
        }
    }

    #[test]
    fn test_bollinger_bands_upper_expected_value() {
        let close = array![
        22.27, 22.19, 22.08, 22.17, 22.18,
        22.13, 22.23, 22.43, 22.24, 22.29,
        22.15, 22.39, 22.38, 22.61, 23.36,
        24.05, 23.75, 23.83, 23.95, 23.63,
        23.82, 23.87, 23.65, 23.19, 23.10,
        23.33, 22.68, 23.10, 22.40, 22.17
    ];

        let input_data = InputData {
            open: None,
            high: None,
            low: None,
            close: Some(close.clone()),
            volume: None,
        };

        let indicator = BBands::new();

        let params = json!({ "period": 20, "std_dev_multiplier": 2.0 });

        let result = indicator.calculate(&input_data, params).unwrap();

        if let OutputData::MultiSeries(output) = result {
            let upper_band = output.get("upper_band").unwrap();

            println!("Upper Band: {:?}", upper_band);


            let invalid_length = 20 - 1;
            for i in invalid_length..close.len() {
                assert!(
                    !upper_band[i].is_nan(),
                    "Expected valid upper band value at index {}, found NaN",
                    i
                );
            }
        } else {
            panic!("Unexpected output format");
        }
    }

    #[test]
    fn test_bollinger_bands_lower_length() {
        let close = array![
        22.27, 22.19, 22.08, 22.17, 22.18,
        22.13, 22.23, 22.43, 22.24, 22.29,
        22.15, 22.39, 22.38, 22.61, 23.36,
        24.05, 23.75, 23.83, 23.95, 23.63,
        23.82, 23.87, 23.65, 23.19, 23.10,
        23.33, 22.68, 23.10, 22.40, 22.17
    ];

        let input_data = InputData {
            open: None,
            high: None,
            low: None,
            close: Some(close.clone()),
            volume: None,
        };

        let indicator = BBands::new();

        let params = json!({ "period": 20, "std_dev_multiplier": 2.0 });

        let result = indicator.calculate(&input_data, params).unwrap();

        if let OutputData::MultiSeries(output) = result {
            let lower_band = output.get("lower_band").unwrap();
            println!("Lower Band: {:?}", lower_band);
            assert_eq!(lower_band.len(), close.len());
        } else {
            panic!("Unexpected output format");
        }
    }

    #[test]
    fn test_bollinger_bands_lower_expected_nan() {
        let close = array![
        22.27, 22.19, 22.08, 22.17, 22.18,
        22.13, 22.23, 22.43, 22.24, 22.29,
        22.15, 22.39, 22.38, 22.61, 23.36,
        24.05, 23.75, 23.83, 23.95, 23.63,
        23.82, 23.87, 23.65, 23.19, 23.10,
        23.33, 22.68, 23.10, 22.40, 22.17
    ];

        let input_data = InputData {
            open: None,
            high: None,
            low: None,
            close: Some(close.clone()),
            volume: None,
        };

        let indicator = BBands::new();

        let params = json!({ "period": 20, "std_dev_multiplier": 2.0 });

        let result = indicator.calculate(&input_data, params).unwrap();

        if let OutputData::MultiSeries(output) = result {
            let lower_band = output.get("lower_band").unwrap();

            println!("Lower Band: {:?}", lower_band);

            let invalid_length = 20 - 1;
            for i in 0..invalid_length {
                assert!(lower_band[i].is_nan(), "Expected NaN at index {}", i);
            }
        } else {
            panic!("Unexpected output format");
        }
    }

    #[test]
    fn test_bollinger_bands_lower_expected_value() {
        let close = array![
        22.27, 22.19, 22.08, 22.17, 22.18,
        22.13, 22.23, 22.43, 22.24, 22.29,
        22.15, 22.39, 22.38, 22.61, 23.36,
        24.05, 23.75, 23.83, 23.95, 23.63,
        23.82, 23.87, 23.65, 23.19, 23.10,
        23.33, 22.68, 23.10, 22.40, 22.17
    ];

        let input_data = InputData {
            open: None,
            high: None,
            low: None,
            close: Some(close.clone()),
            volume: None,
        };

        let indicator = BBands::new();

        let params = json!({ "period": 20, "std_dev_multiplier": 2.0 });

        let result = indicator.calculate(&input_data, params).unwrap();

        if let OutputData::MultiSeries(output) = result {
            let lower_band = output.get("lower_band").unwrap();

            println!("Lower Band: {:?}", lower_band);

            let invalid_length = 20 - 1;
            for i in invalid_length..close.len() {
                assert!(
                    !lower_band[i].is_nan(),
                    "Expected valid lower band value at index {}, found NaN",
                    i
                );
            }
        } else {
            panic!("Unexpected output format");
        }
    }

    #[test]
    fn test_bollinger_bands_deviation_validity() {
        let close = array![
        22.27, 22.19, 22.08, 22.17, 22.18,
        22.13, 22.23, 22.43, 22.24, 22.29,
        22.15, 22.39, 22.38, 22.61, 23.36,
        24.05, 23.75, 23.83, 23.95, 23.63,
        23.82, 23.87, 23.65, 23.19, 23.10,
        23.33, 22.68, 23.10, 22.40, 22.17
    ];

        let input_data = InputData {
            open: None,
            high: None,
            low: None,
            close: Some(close.clone()),
            volume: None,
        };

        let indicator = BBands::new();

        let params = json!({ "period": 20, "std_dev_multiplier": 2.0 });

        let result = indicator.calculate(&input_data, params).unwrap();

        if let OutputData::MultiSeries(output) = result {
            let lower_band = output.get("lower_band").unwrap();
            let middle_band = output.get("middle_band").unwrap();
            let upper_band = output.get("upper_band").unwrap();

            let invalid_length = 20 - 1;

            // Remaining values should be valid numbers
            for i in invalid_length..close.len() {
                assert!(
                    upper_band[i] >= middle_band[i],
                    "Upper band at index {} is not greater than or equal to middle band",
                    i
                );
                assert!(
                    lower_band[i] <= middle_band[i],
                    "Lower band at index {} is not less than or equal to middle band",
                    i
                );
            }
        } else {
            panic!("Unexpected output format");
        }
    }
}