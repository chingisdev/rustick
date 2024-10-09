use std::collections::HashSet;
use ndarray::{s, Array1};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::indicators::utils::{calculate_directional_movements, calculate_true_range, validate_parameter_within_data_length, wilder_smoothing};
use crate::models::data::{BarField, InputData, OutputData};
use crate::models::groups::{CalculationMethodology, ComplexityLevel, DataInputType, Group, MarketSuitability, MathematicalBasis, OutputFormat, SignalInterpretation, SignalType, SmoothingTechnique, TimeframeFocus, TradingStrategySuitability, UseCase};
use crate::models::indicator::{Indicator, IndicatorError};
use crate::validation::validator::{IParameter, ParamRule, Validator};

#[derive(Deserialize, Serialize)]
struct ADXParams {
    #[serde(default = "default_period")]
    period: usize,
}

impl IParameter for ADXParams {}

fn default_period() -> usize {
    14
}


pub struct ADX {
    groups: HashSet<Group>,
    validator: Validator,
}

fn create_groups() -> HashSet<Group> {
    let mut groups = HashSet::new();
    groups.insert(Group::UseCase(UseCase::TrendIdentification));
    groups.insert(Group::UseCase(UseCase::MarketStrengthMeasurement));
    groups.insert(Group::MathematicalBasis(MathematicalBasis::Averaging));
    groups.insert(Group::MathematicalBasis(MathematicalBasis::RatioBased));
    groups.insert(Group::DataInputType(DataInputType::PriceBased));
    groups.insert(Group::SignalType(SignalType::Lagging));
    groups.insert(Group::OutputFormat(OutputFormat::SingleLine));
    groups.insert(Group::OutputFormat(OutputFormat::Percentage));
    groups.insert(Group::TimeframeFocus(TimeframeFocus::Short));
    groups.insert(Group::TimeframeFocus(TimeframeFocus::Medium));
    groups.insert(Group::ComplexityLevel(ComplexityLevel::Intermediate));
    groups.insert(Group::MarketSuitability(MarketSuitability::Trending));
    groups.insert(Group::TradingStrategySuitability(TradingStrategySuitability::Swing));
    groups.insert(Group::TradingStrategySuitability(TradingStrategySuitability::Intraday));
    groups.insert(Group::SmoothingTechnique(SmoothingTechnique::Exponential));
    groups.insert(Group::CalculationMethodology(CalculationMethodology::Ratio));
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

impl ADX {
    pub fn new() -> Self {
        let validator = create_validator();

        let groups = create_groups();

        Self { groups, validator }
    }
}

impl Indicator for ADX {
    fn short_name(&self) -> &'static str {
        "ADX"
    }
    fn name(&self) -> &'static str {
        "Average Directional Index"
    }
    fn get_groups(&mut self) -> &HashSet<Group> {
        &self.groups
    }
    fn calculate(&self, data: &InputData, params: Value) -> Result<OutputData, IndicatorError> {
        // Parse parameters

        let params: ADXParams = serde_json::from_value(params)
            .map_err(|e| IndicatorError::InvalidParameters(e.to_string()))?;

        self.validator.validate(data, &params)?;

        let high = data.get_by_bar_field(&BarField::HIGH).unwrap();
        let low = data.get_by_bar_field(&BarField::LOW).unwrap();
        let close = data.get_by_bar_field(&BarField::CLOSE).unwrap();
        let length = high.len();


        // Step 1: Calculate True Range (TR)
        let tr = calculate_true_range(high, low, close)?;

        // Step 2: Calculate +DM and -DM
        let (plus_dm, minus_dm) = calculate_directional_movements(high, low)?;

        // Step 3: Calculate smoothed TR, +DM, -DM
        let smoothed_tr = wilder_smoothing(&tr, params.period)?;
        let smoothed_plus_dm = wilder_smoothing(&plus_dm, params.period)?;
        let smoothed_minus_dm = wilder_smoothing(&minus_dm, params.period)?;

        // Step 4: Calculate +DI and -DI
        let plus_di = (smoothed_plus_dm / &smoothed_tr) * 100.0;
        let minus_di = (smoothed_minus_dm / &smoothed_tr) * 100.0;

        // Step 5: Calculate DX
        let di_sum = &plus_di + &minus_di;
        let di_diff = (&plus_di - &minus_di).mapv(f64::abs);

        // Handle division by zero
        let dx = di_diff / di_sum * 100.0;
        let dx = dx.mapv(|x| if x.is_nan() || x.is_infinite() { 0.0 } else { x });

        // Step 6: Calculate ADX as the smoothed DX
        let adx = wilder_smoothing(&dx, params.period)?;

        // Initialize the full ADX array with NaNs
        let mut full_adx = Array1::<f64>::from_elem(length, f64::NAN);

        if length < params.period {
            Ok(OutputData::SingleSeries(full_adx))
        } else {
            // Determine the starting index for valid ADX values
            let start_index = 2 * (params.period - 1);

            // Take slices of adx and full_adx starting from start_index
            let valid_adx = adx.slice(s![start_index..]);
            let valid_length = valid_adx.len();

            if start_index + valid_length > length {
                return Err(IndicatorError::CalculationError(
                    "Calculated ADX length exceeds input data length.".to_string(),
                ));
            }

            // Assign valid ADX values to full_adx starting from start_index
            full_adx.slice_mut(s![start_index..start_index + valid_length])
                .assign(&valid_adx);

            Ok(OutputData::SingleSeries(full_adx))
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::data::{InputData, OutputData};
    use serde_json::json;
    use ndarray::{array, Array1};

    #[test]
    fn test_adx() {
        // Sample data
        let high = array![30.1980, 30.3950, 30.4300, 30.6150, 30.7700, 31.1300, 31.2700, 31.1300, 31.2800, 31.4500, 31.4100, 31.4100, 31.3800, 31.5800, 31.7000];
        let low = array![29.8700, 30.0700, 30.2600, 30.3750, 30.5200, 30.8200, 30.8700, 30.7800, 31.0300, 31.2000, 31.0900, 31.1700, 31.0800, 31.3300, 31.4700];
        let close = array![30.0750, 30.2500, 30.3600, 30.5800, 30.6900, 31.0900, 31.1300, 31.1000, 31.2700, 31.3200, 31.2300, 31.3500, 31.2000, 31.5500, 31.6600];
        let length = high.len();
        let input_data = InputData {
            open: None,
            high: Some(high),
            low: Some(low),
            close: Some(close),
            volume: None,
        };

        let indicator = ADX::new();

        let params = json!({ "period": 3 });

        let result = indicator.calculate(&input_data, params).unwrap();

        if let OutputData::SingleSeries(adx_values) = result {
            // Expected results can be calculated from a reliable source or previous calculations
            // For demonstration purposes, we'll check the length and print the values
            println!("ADX values: {:?}", adx_values);

            assert_eq!(adx_values.len(), length);

            // Additional assertions can be added if expected values are known
        } else {
            panic!("Unexpected output format");
        }
    }

    #[test]
    fn test_adx_zero_period() {
        // Sample data
        let high = array![30.0, 31.0, 32.0];
        let low = array![29.0, 30.0, 31.0];
        let close = array![29.5, 30.5, 31.5];

        let input_data = InputData {
            open: None,
            high: Some(high),
            low: Some(low),
            close: Some(close),
            volume: None,
        };

        let indicator = ADX::new();

        let params = json!({ "period": 0 });

        let result = indicator.calculate(&input_data, params);

        assert!(matches!(
            result,
            Err(IndicatorError::InvalidParameters(msg)) if msg == "Parameter 'period' must be a positive integer"
        ));
    }

    #[test]
    fn test_adx_period_greater_than_data_length() {
        // Sample data
        let high = array![30.0, 31.0, 32.0];
        let low = array![29.0, 30.0, 31.0];
        let close = array![29.5, 30.5, 31.5];

        let input_data = InputData {
            open: None,
            high: Some(high),
            low: Some(low),
            close: Some(close),
            volume: None,
        };

        let indicator = ADX::new();

        let params = json!({ "period": 5 });

        let result = indicator.calculate(&input_data, params);
        assert!(matches!(
            result,
            Err(IndicatorError::InvalidParameters(msg)) if msg == "Wrong parameter length. 'period' > data length. (5 > 3)"
        ));
    }

    #[test]
    fn test_adx_data_different_lengths() {
        let high = array![30.0, 31.0, 32.0];
        let low = array![29.0, 30.0]; // Shorter length
        let close = array![29.5, 30.5, 31.5];

        let input_data = InputData {
            open: None,
            high: Some(high),
            low: Some(low),
            close: Some(close),
            volume: None,
        };

        let indicator = ADX::new();

        let params = json!({ "period": 2 });

        let result = indicator.calculate(&input_data, params);

        assert!(matches!(
            result,
            Err(IndicatorError::InvalidInput(msg)) if msg == "Input data series of the bars must have the same length."
        ));
    }

    #[test]
    fn test_adx_missing_high_data() {
        let low = array![29.0, 30.0, 31.0];
        let close = array![29.5, 30.5, 31.5];

        let input_data = InputData {
            open: None,
            high: None, // Missing
            low: Some(low),
            close: Some(close),
            volume: None,
        };

        let indicator = ADX::new();

        let params = json!({ "period": 2 });

        let result = indicator.calculate(&input_data, params);
        assert!(matches!(
            result,
            Err(IndicatorError::InvalidInput(msg)) if msg == "Field 'HIGH' is required but missing."
        ));
    }

    #[test]
    fn test_adx_empty_data() {
        let high = Array1::<f64>::zeros(0);
        let low = Array1::<f64>::zeros(0);
        let close = Array1::<f64>::zeros(0);

        let input_data = InputData {
            open: None,
            high: Some(high),
            low: Some(low),
            close: Some(close),
            volume: None,
        };

        let indicator = ADX::new();

        let params = json!({ "period": 14 });

        let result = indicator.calculate(&input_data, params);
        println!("{:?}", result);
        assert!(matches!(
            result,
            Err(IndicatorError::InvalidParameters(msg)) if msg == "Wrong parameter length. 'period' > data length. (14 > 0)"
        ));
    }

    #[test]
    fn test_adx_with_nan_output() {
        // Sample data
        let high = array![30.0, 31.0, 32.0, 33.0, 34.0];
        let low = array![29.0, 30.0, 31.0, 32.0, 33.0];
        let close = array![29.5, 30.5, 31.5, 32.5, 33.5];

        let input_data = InputData {
            open: None,
            high: Some(high.clone()),
            low: Some(low.clone()),
            close: Some(close.clone()),
            volume: None,
        };

        let indicator = ADX::new();

        let params = json!({ "period": 3 });

        let result = indicator.calculate(&input_data, params).unwrap();

        if let OutputData::SingleSeries(adx_values) = result {
            println!("ADX values: {:?}", adx_values);

            // Assert the length is the same as input
            assert_eq!(adx_values.len(), high.len());

            // The first (2 * (period - 1)) values should be NaN
            let invalid_length = 2 * (3 - 1);
            for i in 0..invalid_length {
                assert!(adx_values[i].is_nan(), "Expected NaN at index {}", i);
            }

            // Remaining values should be valid numbers
            for i in invalid_length..adx_values.len() {
                assert!(
                    !adx_values[i].is_nan(),
                    "Expected valid ADX value at index {}, found NaN",
                    i
                );
            }
        } else {
            panic!("Unexpected output format");
        }
    }

}
