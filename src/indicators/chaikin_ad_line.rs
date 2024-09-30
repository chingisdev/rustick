use std::collections::HashSet;
use serde_json::Value;
use crate::models::data::{InputData, OutputData};
use crate::models::groups::{Group, UseCase, MathematicalBasis, DataInputType, SignalType, OutputFormat, TimeframeFocus, ComplexityLevel, MarketSuitability, TradingStrategySuitability, SmoothingTechnique, CalculationMethodology, SignalInterpretation};
use crate::models::indicator::{Indicator, IndicatorError};

pub struct ChaikinADLine {
    groups: Option<HashSet<Group>>
}

impl Indicator for ChaikinADLine {
    fn short_name(&self) -> &'static str {
        "AD"
    }

    fn name(&self) -> &'static str {
        "Chaikin Accumulation/Distribution Line"
    }

    fn groups(&mut self) -> &HashSet<Group> {
        if self.groups.is_none() {
            let mut groups = HashSet::new();
            groups.insert(Group::UseCase(UseCase::VolumeConfirmation));
            groups.insert(Group::UseCase(UseCase::MarketStrengthMeasurement));
            groups.insert(Group::MathematicalBasis(MathematicalBasis::VolumeWeighted));
            groups.insert(Group::DataInputType(DataInputType::PriceVolumeCombined));
            groups.insert(Group::SignalType(SignalType::Leading));
            groups.insert(Group::OutputFormat(OutputFormat::SingleLine));
            groups.insert(Group::TimeframeFocus(TimeframeFocus::Medium));
            groups.insert(Group::TimeframeFocus(TimeframeFocus::Long));
            groups.insert(Group::ComplexityLevel(ComplexityLevel::Intermediate));
            groups.insert(Group::MarketSuitability(MarketSuitability::Trending));
            groups.insert(Group::TradingStrategySuitability(TradingStrategySuitability::Swing));
            groups.insert(Group::TradingStrategySuitability(TradingStrategySuitability::Positional));
            groups.insert(Group::SmoothingTechnique(SmoothingTechnique::Raw));
            groups.insert(Group::CalculationMethodology(CalculationMethodology::Cumulative));
            groups.insert(Group::SignalInterpretation(SignalInterpretation::Divergence));

            self.groups = Some(groups);
        }
        self.groups.as_ref().unwrap()
    }

    fn calculate(&self, data: &InputData, params: Value) -> Result<OutputData, IndicatorError> {
        let high = data.high.as_ref().ok_or_else(|| {
            IndicatorError::InvalidParameters("Missing High data".to_string())
        })?;
        let low = data.low.as_ref().ok_or_else(|| {
            IndicatorError::InvalidParameters("Missing Low data".to_string())
        })?;
        let close = data.close.as_ref().ok_or_else(|| {
            IndicatorError::InvalidParameters("Missing Close data".to_string())
        })?;
        let volume = data.volume.as_ref().ok_or_else(|| {
            IndicatorError::InvalidParameters("Missing Volume data".to_string())
        })?;

        let length = high.len();

        if low.len() != length || close.len() != length || volume.len() != length {
            return Err(IndicatorError::InvalidParameters("Input data lengths do not match".to_string()));
        }

        let mut ad_line = Vec::with_capacity(length);
        let mut cumulative_ad = 0.0;
        for i in 0..length {
            let high_low_range = high[i] - low[i];
            if high_low_range == 0.0 {
                ad_line.push(cumulative_ad);
                continue;
            }
            cumulative_ad += ((close[i] - low[i]) - (high[i] - close[i])) / high_low_range * volume[i];
            ad_line.push(cumulative_ad);
        }

        Ok(OutputData::SingleSeries(ad_line))

    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::data::{InputData, OutputData};
    use serde_json::Value;

    #[test]
    fn test_chaikin_ad_line() {
        // Sample data
        let high = vec![10.0, 11.0, 12.0, 13.0, 14.0];
        let low = vec![9.0, 9.5, 10.5, 11.5, 12.5];
        let close = vec![9.5, 10.5, 11.5, 12.5, 13.5];
        let volume = vec![1000.0, 1100.0, 1200.0, 1300.0, 1400.0];

        let input_data = InputData {
            open: Some(vec![]),
            high: Some(high),
            low: Some(low),
            close: Some(close),
            volume: Some(volume),
        };

        let indicator = ChaikinADLine { groups: None };

        // No parameters are needed for Chaikin A/D Line
        let params = Value::Null;

        let result = indicator.calculate(&input_data, params).unwrap();

        if let OutputData::SingleSeries(ad_line) = result {
            // Expected results calculated manually
            let expected = vec![
                0.0,        // Day 1
                366.6667,   // Day 2
                766.6667,   // Day 3
                1200.0,     // Day 4
                1666.6667,  // Day 5
            ];

            // Compare the calculated A/D Line with expected results
            for (calculated, expected) in ad_line.iter().zip(expected.iter()) {
                assert!(
                    (calculated - expected).abs() < 0.0001,
                    "Calculated value {} does not match expected value {}",
                    calculated,
                    expected
                );
            }
        } else {
            panic!("Unexpected output format");
        }
    }
}