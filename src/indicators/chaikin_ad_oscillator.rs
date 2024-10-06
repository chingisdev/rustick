use std::collections::HashSet;
use ndarray::s;
use serde_json::Value;
use serde::{Deserialize, Serialize};
use crate::indicators::utils::{calculate_adl, calculate_ema, validate_period_less_than_data};
use crate::models::data::{BarField, InputData, OutputData};
use crate::models::groups::{CalculationMethodology, ComplexityLevel, DataInputType, Group, MarketSuitability, MathematicalBasis, OutputFormat, SignalInterpretation, SignalType, SmoothingTechnique, TimeframeFocus, TradingStrategySuitability, UseCase};
use crate::models::indicator::{Indicator, IndicatorError};
use crate::validation::validator::{IParameter, ParamRule, Validator};

#[derive(Deserialize, Serialize)]
struct ChaikinOscillatorParams {
    #[serde(default = "default_short_period")]
    short_period: usize,
    #[serde(default = "default_long_period")]
    long_period: usize,
}

impl IParameter for ChaikinOscillatorParams {}

fn default_short_period() -> usize {
    3
}

fn default_long_period() -> usize {
    10
}


pub struct ChaikinADOscillator {
    groups: HashSet<Group>,
    validator: Validator,
}

fn create_validator() -> Validator {
    Validator::new(
        vec![
            BarField::HIGH, BarField::LOW, BarField::CLOSE, BarField::VOLUME
        ],
        vec![
            ParamRule::Required("short_period"),
            ParamRule::Required("long_period"),
            ParamRule::PositiveInteger("short_period"),
            ParamRule::PositiveInteger("long_period"),
            ParamRule::CorrectPeriod {left: "short_period", right: "long_period"},
            ParamRule::Custom(Box::new(|value: &Value, data: &InputData| validate_period_less_than_data(value, data, "short_period", BarField::HIGH))),
            ParamRule::Custom(Box::new(|value: &Value, data: &InputData| validate_period_less_than_data(value, data, "long_period", BarField::HIGH))),
        ],
    )
}

impl ChaikinADOscillator {
    pub fn new() -> Self {
        let groups = create_groups();
        let validator = create_validator();
        Self { groups, validator }
    }
}

fn create_groups() -> HashSet<Group> {
    let mut groups = HashSet::new();
    groups.insert(Group::UseCase(UseCase::MomentumDetection));
    groups.insert(Group::UseCase(UseCase::VolumeConfirmation));
    groups.insert(Group::MathematicalBasis(MathematicalBasis::Differentiation));
    groups.insert(Group::MathematicalBasis(MathematicalBasis::VolumeWeighted));
    groups.insert(Group::DataInputType(DataInputType::PriceVolumeCombined));
    groups.insert(Group::SignalType(SignalType::Leading));
    groups.insert(Group::OutputFormat(OutputFormat::SingleLine));
    groups.insert(Group::TimeframeFocus(TimeframeFocus::Short));
    groups.insert(Group::TimeframeFocus(TimeframeFocus::Medium));
    groups.insert(Group::ComplexityLevel(ComplexityLevel::Intermediate));
    groups.insert(Group::MarketSuitability(MarketSuitability::Trending));
    groups.insert(Group::MarketSuitability(MarketSuitability::RangeBound));
    groups.insert(Group::TradingStrategySuitability(TradingStrategySuitability::Intraday));
    groups.insert(Group::TradingStrategySuitability(TradingStrategySuitability::Swing));
    groups.insert(Group::SmoothingTechnique(SmoothingTechnique::Exponential));
    groups.insert(Group::CalculationMethodology(CalculationMethodology::Differential));
    groups.insert(Group::SignalInterpretation(SignalInterpretation::Crossovers));
    groups.insert(Group::SignalInterpretation(SignalInterpretation::Divergence));
    groups
}

impl Indicator for ChaikinADOscillator {
    fn short_name(&self) -> &'static str {
        "ADOSC"
    }

    fn name(&self) -> &'static str {
        "Chaikin Accumulation/Distribution Oscillator"
    }

    fn get_groups(&mut self) -> &HashSet<Group> {
        &self.groups
    }

    fn calculate(&self, data: &InputData, params: Value) -> Result<OutputData, IndicatorError> {
        let params: ChaikinOscillatorParams = serde_json::from_value(params)
            .map_err(|e| IndicatorError::InvalidParameters(e.to_string()))?;

        self.validator.validate(data, &params)?;

        let high = data.get_by_bar_field(&BarField::HIGH).unwrap();
        let low = data.get_by_bar_field(&BarField::LOW).unwrap();
        let close = data.get_by_bar_field(&BarField::CLOSE).unwrap();
        let volume = data.get_by_bar_field(&BarField::VOLUME).unwrap();

        // Step 1: Calculate the Accumulation/Distribution Line (ADL)
        let adl = calculate_adl(high, low, close, volume)?;

        // Step 2: Calculate EMAs of the ADL
        let short_ema = calculate_ema(&adl, params.short_period)?;
        let long_ema = calculate_ema(&adl, params.long_period)?;

        let start_index = params.long_period - 1;
        let oscillator_values = &short_ema.slice(s![start_index..]) - &long_ema.slice(s![start_index..]);

        Ok(OutputData::SingleSeries(oscillator_values.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::data::{InputData, OutputData};
    use serde_json::json;
    use ndarray::{array, Array1};

    #[test]
    fn test_chaikin_oscillator() {
        // Sample data
        let high = array![10.0, 11.0, 12.0, 13.0, 14.0, 15.0, 16.0, 17.0, 18.0, 19.0];
        let low = array![9.0, 9.5, 10.5, 11.5, 12.5, 13.5, 14.5, 15.5, 16.5, 17.5];
        let close = array![9.5, 10.5, 11.5, 12.5, 13.5, 14.5, 15.5, 16.5, 17.5, 18.5];
        let volume = array![1000.0, 1100.0, 1200.0, 1300.0, 1400.0, 1500.0, 1600.0, 1700.0, 1800.0, 1900.0];

        let input_data = InputData {
            open: None,
            high: Some(high),
            low: Some(low),
            close: Some(close),
            volume: Some(volume),
        };

        let indicator = ChaikinADOscillator::new();

        // Use default parameters
        let params = json!({});

        let result = indicator.calculate(&input_data, params).unwrap();
        println!("{:?}", result);
        if let OutputData::SingleSeries(chaikin_osc) = result {
            // Expected results would be calculated from a trusted source or precomputed
            // For demonstration, we'll print the values
            println!("Chaikin Oscillator values: {:?}", chaikin_osc);

            // Since we don't have precomputed expected values, we can check the length
            let expected_length = input_data.high.as_ref().unwrap().len() - (10 - 1); // long_period - 1
            assert_eq!(chaikin_osc.len(), expected_length);

            // Further assertions can be added with known expected values
        } else {
            panic!("Unexpected output format");
        }
    }

    #[test]
    fn test_chaikin_oscillator_zero_short_period() {
        // Sample data
        let high = array![10.0, 11.0, 12.0, 13.0, 14.0];
        let low = array![9.0, 10.0, 11.0, 12.0, 13.0];
        let close = array![9.5, 10.5, 11.5, 12.5, 13.5];
        let volume = array![1000.0, 5.0, 12.0, 13.0, 14.0];

        let input_data = InputData {
            open: None,
            high: Some(high),
            low: Some(low),
            close: Some(close),
            volume: Some(volume),
        };

        let indicator = ChaikinADOscillator::new();

        // Set short_period to zero
        let params = json!({ "short_period": 0, "long_period": 3 });

        let result = indicator.calculate(&input_data, params);
        println!("{:?}", result);
        assert!(matches!(result, Err(IndicatorError::InvalidParameters(msg)) if msg == "Parameter 'short_period' must be a positive integer"));
    }

    #[test]
    fn test_chaikin_oscillator_zero_long_period() {
        // Sample data
        let high = array![10.0, 11.0, 12.0];
        let low = array![9.0, 10.0, 11.0];
        let close = array![9.5, 10.5, 11.5];
        let volume = array![1000.0, 3.0, 5.0];

        let input_data = InputData {
            open: None,
            high: Some(high),
            low: Some(low),
            close: Some(close),
            volume: Some(volume),
        };

        let indicator = ChaikinADOscillator::new();

        // Set long_period to zero
        let params = json!({ "short_period": 2, "long_period": 0 });

        let result = indicator.calculate(&input_data, params);
        println!("{:?}", result);
        assert!(matches!(
        result,
        Err(IndicatorError::InvalidParameters(msg)) if msg == "Parameter 'long_period' must be a positive integer"
    ));
    }

    #[test]
    fn test_chaikin_oscillator_short_period_greater_or_equal_long_period() {
        // Sample data
        let high = array![10.0, 11.0, 12.0, 13.0];
        let low = array![9.0, 10.0, 11.0, 12.0];
        let close = array![9.5, 10.5, 11.5, 12.5];
        let volume = array![1000.0, 4.0, 12.0, 13.0];

        let input_data = InputData {
            open: None,
            high: Some(high),
            low: Some(low),
            close: Some(close),
            volume: Some(volume),
        };

        let indicator = ChaikinADOscillator::new();

        // Set short_period equal to long_period
        let params = json!({ "short_period": 3, "long_period": 3 });

        let result = indicator.calculate(&input_data, params);
        println!("{:?}", result);
        assert!(matches!(
            result,
            Err(IndicatorError::InvalidParameters(msg)) if msg == "Parameter 'short_period' must be less than 'long_period'"
        ));
    }

    #[test]
    fn test_chaikin_oscillator_short_period_greater_than_data_length() {
        // Sample data
        let high = array![10.0, 11.0, 12.0];
        let low = array![9.0, 10.0, 11.0];
        let close = array![9.5, 10.5, 11.5];
        let volume = array![1000.0, 3.0, 200.0];

        let input_data = InputData {
            open: None,
            high: Some(high),
            low: Some(low),
            close: Some(close),
            volume: Some(volume),
        };

        let indicator = ChaikinADOscillator::new();

        // Set short_period greater than data length
        let params = json!({ "short_period": 5, "long_period": 6 });

        let result = indicator.calculate(&input_data, params);
        println!("{:?}", result);
        assert!(matches!(
            result,
            Err(IndicatorError::InvalidParameters(msg)) if msg == "Wrong parameter length. 'short_period' > data length. (5 > 3)"
        ));
    }

    #[test]
    fn test_chaikin_oscillator_long_period_greater_than_data_length() {
        // Sample data
        let open = array![10.0, 11.0, 12.0];
        let high = array![10.0, 11.0, 12.0];
        let low = array![9.0, 10.0, 11.0];
        let close = array![9.5, 10.5, 11.5];
        let volume = array![1000.0, 3.0, 11.5];

        let input_data = InputData {
            open: Some(open),
            high: Some(high),
            low: Some(low),
            close: Some(close),
            volume: Some(volume),
        };

        let indicator = ChaikinADOscillator::new();

        // Set long_period greater than data length
        let params = json!({ "short_period": 2, "long_period": 5 });

        let result = indicator.calculate(&input_data, params);
        println!("{:?}", result);
        assert!(matches!(
            result,
            Err(IndicatorError::InvalidParameters(msg)) if msg == "Wrong parameter length. 'long_period' > data length. (5 > 3)"
        ));
    }

    #[test]
    fn test_chaikin_oscillator_data_different_lengths() {
        // Data arrays of different lengths
        let open = array![10.0, 11.0, 12.0];
        let high = array![10.0, 11.0, 12.0, 13.0];
        let low = array![9.0, 10.0, 11.0]; // Shorter length
        let close = array![9.5, 10.5, 11.5, 12.5];
        let volume = array![1000.0, 4.0];

        let input_data = InputData {
            open: Some(open),
            high: Some(high),
            low: Some(low),
            close: Some(close),
            volume: Some(volume),
        };

        let indicator = ChaikinADOscillator::new();

        let params = json!({});

        let result = indicator.calculate(&input_data, params);
        println!("{:?}", result);
        assert!(matches!(
            result,
            Err(IndicatorError::InvalidInput(msg)) if msg == "Input data series of the bars must have the same length."
        ));
    }


    #[test]
    fn test_chaikin_oscillator_missing_high_data() {
        // Missing high data
        let low = array![9.0, 10.0, 11.0, 12.0];
        let close = array![9.5, 10.5, 11.5, 12.5];
        let volume = array![1000.0, 4.0];

        let input_data = InputData {
            open: None,
            high: None, // Missing
            low: Some(low),
            close: Some(close),
            volume: Some(volume),
        };

        let indicator = ChaikinADOscillator::new();

        let params = json!({});

        let result = indicator.calculate(&input_data, params);
        println!("{:?}", result);
        assert!(matches!(
            result,
            Err(IndicatorError::InvalidInput(msg)) if msg == "Field 'HIGH' is required but missing."
        ));
    }

    #[test]
    fn test_chaikin_oscillator_missing_low_data() {
        // Missing low data
        let high = array![10.0, 11.0, 12.0, 13.0];
        let close = array![9.5, 10.5, 11.5, 12.5];
        let volume = array![1000.0, 4.0];

        let input_data = InputData {
            open: None,
            high: Some(high),
            low: None, // Missing
            close: Some(close),
            volume: Some(volume),
        };

        let indicator = ChaikinADOscillator::new();

        let params = json!({});

        let result = indicator.calculate(&input_data, params);
        println!("{:?}", result);
        assert!(matches!(
            result,
            Err(IndicatorError::InvalidInput(msg)) if msg == "Field 'LOW' is required but missing."
        ));
    }

    #[test]
    fn test_chaikin_oscillator_missing_close_data() {
        // Missing close data
        let high = array![10.0, 11.0, 12.0, 13.0];
        let low = array![9.0, 10.0, 11.0, 12.0];
        let volume = array![1000.0, 4.0];

        let input_data = InputData {
            open: None,
            high: Some(high),
            low: Some(low),
            close: None, // Missing
            volume: Some(volume),
        };

        let indicator = ChaikinADOscillator::new();

        let params = json!({});

        let result = indicator.calculate(&input_data, params);
        println!("{:?}", result);
        assert!(matches!(
            result,
            Err(IndicatorError::InvalidInput(msg)) if msg == "Field 'CLOSE' is required but missing."
        ));
    }

    #[test]
    fn test_chaikin_oscillator_missing_volume_data() {
        // Missing volume data
        let high = array![10.0, 11.0, 12.0, 13.0];
        let low = array![9.0, 10.0, 11.0, 12.0];
        let close = array![9.5, 10.5, 11.5, 12.5];

        let input_data = InputData {
            open: None,
            high: Some(high),
            low: Some(low),
            close: Some(close),
            volume: None, // Missing
        };

        let indicator = ChaikinADOscillator::new();

        let params = json!({});

        let result = indicator.calculate(&input_data, params);
        println!("{:?}", result);
        assert!(matches!(
            result,
            Err(IndicatorError::InvalidInput(msg)) if msg == "Field 'VOLUME' is required but missing."
        ));
    }

    #[test]
    fn test_chaikin_oscillator_insufficient_data_length() {
        // Data length less than long_period
        let high = array![10.0, 11.0];
        let low = array![9.0, 10.0];
        let close = array![9.5, 10.5];
        let volume = array![1000.0, 2.0];

        let input_data = InputData {
            open: None,
            high: Some(high),
            low: Some(low),
            close: Some(close),
            volume: Some(volume),
        };

        let indicator = ChaikinADOscillator::new();

        // Default periods are 3 and 10, but data length is 2
        let params = json!({});

        let result = indicator.calculate(&input_data, params);
        println!("{:?}", result);
        assert!(matches!(
            result,
            Err(IndicatorError::InvalidParameters(msg)) if msg == "Wrong parameter length. 'short_period' > data length. (3 > 2)"
        ));
    }

    #[test]
    fn test_chaikin_oscillator_empty_data() {
        // Empty data arrays
        let high = Array1::<f64>::zeros(0);
        let low = Array1::<f64>::zeros(0);
        let close = Array1::<f64>::zeros(0);
        let volume = Array1::<f64>::zeros(0);

        let input_data = InputData {
            open: None,
            high: Some(high),
            low: Some(low),
            close: Some(close),
            volume: Some(volume),
        };

        let indicator = ChaikinADOscillator::new();

        let params = json!({});

        let result = indicator.calculate(&input_data, params);
        println!("{:?}", result);
        assert!(matches!(
            result,
            Err(IndicatorError::InvalidParameters(msg)) if msg == "Wrong parameter length. 'short_period' > data length. (3 > 0)"
        ));
    }
}
