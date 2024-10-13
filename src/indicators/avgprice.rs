use std::collections::HashSet;
use serde_json::Value;
use crate::models::data::{BarField, InputData, OutputData};
use crate::models::groups::{CalculationMethodology, ComplexityLevel, DataInputType, Group, MarketSuitability, MathematicalBasis, OutputFormat, SignalInterpretation, SignalType, SmoothingTechnique, TimeframeFocus, TradingStrategySuitability, UseCase};
use crate::models::indicator::{Indicator, IndicatorError};
use crate::validation::validator::Validator;

pub struct AvgPrice {
    groups: HashSet<Group>,
    validator: Validator,
}

fn create_groups() -> HashSet<Group> {
    let mut groups = HashSet::new();
    groups.insert(Group::UseCase(UseCase::PriceTransformation));
    groups.insert(Group::MathematicalBasis(MathematicalBasis::Averaging));
    groups.insert(Group::DataInputType(DataInputType::PriceBased));
    groups.insert(Group::SignalType(SignalType::Coincident));
    groups.insert(Group::OutputFormat(OutputFormat::SingleLine));
    groups.insert(Group::OutputFormat(OutputFormat::Absolute));
    groups.insert(Group::TimeframeFocus(TimeframeFocus::Short));
    groups.insert(Group::TimeframeFocus(TimeframeFocus::Medium));
    groups.insert(Group::TimeframeFocus(TimeframeFocus::Long));
    groups.insert(Group::ComplexityLevel(ComplexityLevel::Basic));
    groups.insert(Group::MarketSuitability(MarketSuitability::Trending));
    groups.insert(Group::MarketSuitability(MarketSuitability::RangeBound));
    groups.insert(Group::TradingStrategySuitability(TradingStrategySuitability::Intraday));
    groups.insert(Group::TradingStrategySuitability(TradingStrategySuitability::Swing));
    groups.insert(Group::TradingStrategySuitability(TradingStrategySuitability::Positional));
    groups.insert(Group::SmoothingTechnique(SmoothingTechnique::Raw));
    groups.insert(Group::CalculationMethodology(CalculationMethodology::Averaging));
    groups.insert(Group::SignalInterpretation(SignalInterpretation::Patterns));

    groups
}

fn create_validator() -> Validator {
    Validator::new(
        vec![BarField::CLOSE, BarField::LOW, BarField::HIGH, BarField::OPEN],
        vec![]
    )
}

impl AvgPrice {
    pub fn new() -> Self {
        let groups = create_groups();
        let validator = create_validator();
        Self { groups, validator }
    }
}

impl Indicator for AvgPrice {
    fn short_name(&self) -> &'static str {
        "AVGPRICE"
    }

    fn name(&self) -> &'static str {
        "Average Price"
    }

    fn get_groups(&mut self) -> &HashSet<Group> {
        &self.groups
    }

    fn calculate(&self, data: &InputData, params: Value) -> Result<OutputData, IndicatorError> {
        self.validator.validate_data(data)?;

        let open = data.get_by_bar_field(&BarField::OPEN).unwrap();
        let high = data.get_by_bar_field(&BarField::HIGH).unwrap();
        let low = data.get_by_bar_field(&BarField::LOW).unwrap();
        let close = data.get_by_bar_field(&BarField::CLOSE).unwrap();
        let sum = open + high + low + close;
        let avg_price = sum / 4.0;
        Ok(OutputData::SingleSeries(avg_price))
    }
}

#[cfg(test)]
mod test {
    use ndarray::array;
    use serde_json::json;
    use super::*;

    #[test]
    fn test_avg_price_length() {
        // Sample open, high, low, and close price data
        let open = array![1.0, 2.0, 3.0, 4.0, 5.0];
        let high = array![1.5, 2.5, 3.5, 4.5, 5.5];
        let low = array![0.5, 1.5, 2.5, 3.5, 4.5];
        let close = array![1.2, 2.2, 3.2, 4.2, 5.2];

        let input_data = InputData {
            open: Some(open.clone()),
            high: Some(high.clone()),
            low: Some(low.clone()),
            close: Some(close.clone()),
            volume: None,
        };

        let indicator = AvgPrice::new();

        let params = json!({}); // No parameters needed

        let result = indicator.calculate(&input_data, params).unwrap();

        if let OutputData::SingleSeries(avg_price_values) = result {
            println!("Average Price values: {:?}", avg_price_values);

            // Assert the length is the same as input
            assert_eq!(avg_price_values.len(), open.len());
        } else {
            panic!("Unexpected output format");
        }
    }

    #[test]
    fn test_avg_price_expected_value() {
        // Sample open, high, low, and close price data
        let open = array![1.0, 2.0, 3.0, 4.0, 5.0];
        let high = array![1.5, 2.5, 3.5, 4.5, 5.5];
        let low = array![0.5, 1.5, 2.5, 3.5, 4.5];
        let close = array![1.2, 2.2, 3.2, 4.2, 5.2];

        let input_data = InputData {
            open: Some(open.clone()),
            high: Some(high.clone()),
            low: Some(low.clone()),
            close: Some(close.clone()),
            volume: None,
        };

        let indicator = AvgPrice::new();

        let params = json!({}); // No parameters needed

        let result = indicator.calculate(&input_data, params).unwrap();

        if let OutputData::SingleSeries(avg_price_values) = result {
            println!("Average Price values: {:?}", avg_price_values);
            // Calculate expected values manually and compare
            let expected_avg_price = (&open + &high + &low + &close) / 4.0;

            for i in 0..open.len() {
                assert!(
                    (avg_price_values[i] - expected_avg_price[i]).abs() < 1e-6,
                    "Average Price value at index {} does not match expected value. {} and {}",
                    i, avg_price_values[i], expected_avg_price[i]
                );
            }
        } else {
            panic!("Unexpected output format");
        }
    }

}