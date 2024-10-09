use std::collections::HashSet;
use ndarray::Array1;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use crate::indicators::adx::ADX;
use crate::indicators::utils::validate_parameter_within_data_length;
use crate::models::data::{BarField, InputData, OutputData};
use crate::models::groups::{CalculationMethodology, ComplexityLevel, DataInputType, Group, MarketSuitability, MathematicalBasis, OutputFormat, SignalInterpretation, SignalType, SmoothingTechnique, TimeframeFocus, TradingStrategySuitability, UseCase};
use crate::models::indicator::{Indicator, IndicatorError};
use crate::validation::validator::{IParameter, ParamRule, Validator};

#[derive(Serialize, Deserialize)]
pub struct ADXRParams {
    #[serde(default = "default_period")]
    pub period: usize,
}

fn default_period() -> usize { 14 }

impl IParameter for ADXRParams {}

pub struct ADXR {
    groups: HashSet<Group>,
    validator: Validator,
}

fn create_groups() -> HashSet<Group> {
    let mut groups = HashSet::new();
    groups.insert(Group::UseCase(UseCase::TrendIdentification));
    groups.insert(Group::UseCase(UseCase::MarketStrengthMeasurement));
    groups.insert(Group::MathematicalBasis(MathematicalBasis::Averaging));
    groups.insert(Group::MathematicalBasis(MathematicalBasis::RatioBased));
    groups.insert(Group::MathematicalBasis(MathematicalBasis::Oscillation));
    groups.insert(Group::DataInputType(DataInputType::PriceBased));
    groups.insert(Group::SignalType(SignalType::Lagging));
    groups.insert(Group::OutputFormat(OutputFormat::SingleLine));
    groups.insert(Group::OutputFormat(OutputFormat::Percentage));
    groups.insert(Group::TimeframeFocus(TimeframeFocus::Medium));
    groups.insert(Group::TimeframeFocus(TimeframeFocus::Long));
    groups.insert(Group::ComplexityLevel(ComplexityLevel::Intermediate));
    groups.insert(Group::MarketSuitability(MarketSuitability::Trending));
    groups.insert(Group::TradingStrategySuitability(TradingStrategySuitability::Swing));
    groups.insert(Group::TradingStrategySuitability(TradingStrategySuitability::Positional));
    groups.insert(Group::SmoothingTechnique(SmoothingTechnique::SimpleAverage));
    groups.insert(Group::CalculationMethodology(CalculationMethodology::Ratio));
    groups.insert(Group::CalculationMethodology(CalculationMethodology::Cumulative));
    groups.insert(Group::SignalInterpretation(SignalInterpretation::ThresholdLevels));
    groups
}

fn create_validator() -> Validator {
    Validator::new(
        vec![BarField::HIGH, BarField::LOW, BarField::CLOSE],
        vec![
            ParamRule::Required("period"),
            ParamRule::PositiveInteger("period"),
            ParamRule::Custom(Box::new(|value: &Value, data: &InputData| validate_parameter_within_data_length(value, data, "period", BarField::HIGH))),
        ]
    )
}

impl ADXR {
    pub fn new() -> Self {
        let groups = create_groups();
        let validator = create_validator();
        Self { groups, validator }
    }
}

impl Indicator for ADXR {
    fn short_name(&self) -> &'static str {
        "ADXR"
    }

    fn name(&self) -> &'static str {
        "Average Directional Movement Index Rating"
    }

    fn get_groups(&mut self) -> &HashSet<Group> {
        &self.groups
    }

    fn calculate(&self, data: &InputData, params: Value) -> Result<OutputData, IndicatorError> {
        let adxr_params: ADXRParams = serde_json::from_value(params.clone()).map_err(|e| IndicatorError::InvalidParameters(e.to_string()))?;
        self.validator.validate(data, &adxr_params)?;
        let adx_indicator = ADX::new();
        let adx_result = adx_indicator.calculate(data, params)?;

        let adx_values = match adx_result {
            OutputData::SingleSeries(series) => series,
            _ => return Err(IndicatorError::CalculationError("Invalid ADX output.".to_string())),
        };

        let length = adx_values.len();
        // Initialize ADXR array with NaNs
        let mut adxr_values = Array1::<f64>::from_elem(length, f64::NAN);

        // Compute ADXR
        for i in adxr_params.period..adxr_values.len() {
            if adx_values[i].is_nan() || adx_values[i - adxr_params.period].is_nan() {
                continue;
            }
            adxr_values[i] = (adx_values[i] + adx_values[i - adxr_params.period]) / 2.0;
        }

        Ok(OutputData::SingleSeries(adxr_values))
    }
}

#[cfg(test)]
mod tests {
    use ndarray::array;
    use super::*;
    use crate::models::data::InputData;

    #[test]
    fn test_adxr() {
        // Sample data (you can use real market data for more accurate testing)
        let high = array![30.0, 32.0, 31.0, 33.0, 34.0, 35.0, 36.0, 37.0, 38.0, 39.0];
        let low = array![29.0, 30.0, 29.5, 31.0, 32.0, 33.0, 34.0, 35.0, 36.0, 37.0];
        let close = array![29.5, 31.0, 30.5, 32.0, 33.0, 34.0, 35.0, 36.0, 37.0, 38.0];

        let input_data = InputData {
            open: None,
            high: Some(high.clone()),
            low: Some(low.clone()),
            close: Some(close.clone()),
            volume: None,
        };

        let indicator = ADXR::new();
        let period_num = 3;
        let params = json!({ "period": period_num });

        let result = indicator.calculate(&input_data, params).unwrap();

        if let OutputData::SingleSeries(adxr_values) = result {
            println!("ADXR values: {:?}", adxr_values);

            // Assert the length is the same as input
            assert_eq!(adxr_values.len(), high.len());

            // The first (2 * (period - 1)) values should be NaN
            let invalid_length = 2 * (period_num - 1) + period_num;
            for i in 0..invalid_length {
                assert!(adxr_values[i].is_nan(), "Expected NaN at index {}", i);
            }

            // Remaining values should be valid numbers
            for i in invalid_length..adxr_values.len() {
                assert!(
                    !adxr_values[i].is_nan(),
                    "Expected valid ADXR value at index {}, found NaN",
                    i
                );
            }
        } else {
            panic!("Unexpected output format");
        }
    }
}