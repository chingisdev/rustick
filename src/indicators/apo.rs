use std::collections::HashSet;
use ndarray::{s, Array1};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use crate::indicators::utils::validate_period_less_than_data;
use crate::models::data::{BarField, InputData, OutputData};
use crate::models::groups::{CalculationMethodology, ComplexityLevel, DataInputType, Group, MarketSuitability, MathematicalBasis, OutputFormat, SignalInterpretation, SignalType, SmoothingTechnique, TimeframeFocus, TradingStrategySuitability, UseCase};
use crate::models::indicator::{Indicator, IndicatorError};
use crate::validation::validator::{IParameter, ParamRule, Validator};

#[derive(Serialize, Deserialize)]
pub struct APOParams {
    #[serde(default = "default_fast_period")]
    pub fast_period: usize,
    #[serde(default = "default_slow_period")]
    pub slow_period: usize,
}

fn default_fast_period() -> usize { 12 }
fn default_slow_period() -> usize { 26 }

impl IParameter for APOParams {}

pub struct APO {
    groups: HashSet<Group>,
    validator: Validator
}

fn create_groups() -> HashSet<Group> {
    let mut groups = HashSet::new();
    groups.insert(Group::UseCase(UseCase::MomentumDetection));
    groups.insert(Group::UseCase(UseCase::TrendIdentification));
    groups.insert(Group::MathematicalBasis(MathematicalBasis::Differentiation));
    groups.insert(Group::MathematicalBasis(MathematicalBasis::Oscillation));
    groups.insert(Group::DataInputType(DataInputType::PriceBased));
    groups.insert(Group::SignalType(SignalType::Lagging));
    groups.insert(Group::OutputFormat(OutputFormat::SingleLine));
    groups.insert(Group::TimeframeFocus(TimeframeFocus::Short));
    groups.insert(Group::TimeframeFocus(TimeframeFocus::Medium));
    groups.insert(Group::ComplexityLevel(ComplexityLevel::Basic));
    groups.insert(Group::MarketSuitability(MarketSuitability::Trending));
    groups.insert(Group::MarketSuitability(MarketSuitability::RangeBound));
    groups.insert(Group::TradingStrategySuitability(TradingStrategySuitability::Intraday));
    groups.insert(Group::TradingStrategySuitability(TradingStrategySuitability::Swing));
    groups.insert(Group::SmoothingTechnique(SmoothingTechnique::Exponential));
    groups.insert(Group::CalculationMethodology(CalculationMethodology::Differential));
    groups.insert(Group::SignalInterpretation(SignalInterpretation::Crossovers));
    groups
}

fn create_validator() -> Validator {
    Validator::new(
        vec![BarField::CLOSE],
        vec![
            ParamRule::Required("fast_period"),
            ParamRule::Required("slow_period"),
            ParamRule::PositiveInteger("fast_period"),
            ParamRule::PositiveInteger("slow_period"),
            ParamRule::CorrectPeriod {left: "fast_period", right: "slow_period"},
            ParamRule::Custom(Box::new(|value: &Value, data: &InputData| validate_period_less_than_data(value, data, "fast_period", BarField::CLOSE))),
            ParamRule::Custom(Box::new(|value: &Value, data: &InputData| validate_period_less_than_data(value, data, "slow_period", BarField::CLOSE))),
        ]
    )
}

impl APO {
    pub fn new() -> Self {
        let groups = create_groups();
        let validator = create_validator();
        APO { groups, validator }
    }
}

impl Indicator for APO {
    fn short_name(&self) -> &'static str {
        "APO"
    }

    fn name(&self) -> &'static str {
        "Absolute Price Oscillator"
    }

    fn get_groups(&mut self) -> &HashSet<Group> {
        &self.groups
    }

    fn calculate(&self, data: &InputData, params: Value) -> Result<OutputData, IndicatorError> {
        let params: APOParams = serde_json::from_value(params)
            .map_err(|e| IndicatorError::InvalidParameters(e.to_string()))?;

        self.validator.validate(data, &params)?;

        let close = data.get_by_bar_field(&BarField::CLOSE).unwrap();

        let fast_ema = exponential_moving_average(close, params.fast_period);
        let slow_ema = exponential_moving_average(close, params.slow_period);

        let apo_values = fast_ema - slow_ema;

        Ok(OutputData::SingleSeries(apo_values))
    }
}

pub fn exponential_moving_average(
    data: &Array1<f64>,
    period: usize,
) -> Array1<f64> {
    let length = data.len();
    let mut ema = Array1::<f64>::from_elem(length, f64::NAN);
    let multiplier = 2.0 / (period as f64 + 1.0);

    let initial_sma = data.slice(s![..period]).mean().unwrap();
    ema[period - 1] = initial_sma;

    for i in period..length {
        ema[i] = (data[i] - ema[i - 1]) * multiplier + ema[i - 1];
    }

    ema
}
