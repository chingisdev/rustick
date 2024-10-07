use ndarray::Array1;
use serde::Serialize;
use serde_json::Value;
use crate::models::data::{BarField, InputData};
use crate::models::indicator::IndicatorError;

pub struct Validator {
    candle_validator: CandleValidator,
    parameter_validator: ParameterValidator
}

pub struct CandleValidator {
    pub required_fields: Vec<BarField>,
}

pub struct ParameterValidator {
    pub param_rules: Vec<ParamRule>,
}

pub trait IParameter {
    fn to_value(&self) -> Value where Self: Serialize {
        serde_json::to_value(self).unwrap()
    }
}

impl CandleValidator {
    fn is_field_missing(data: &InputData, field: &BarField) -> bool {
        data.get_by_bar_field(field).is_none()
    }

    fn validate_required_fields_presence(&self, data: &InputData) -> Result<(), IndicatorError> {
        for field in &self.required_fields {
            if CandleValidator::is_field_missing(data, &field) {
                let field_missing_error = format!("Field '{}' is required but missing.", &field.to_str());
                return Err(IndicatorError::InvalidInput(field_missing_error));
            }
        }

        Ok(())
    }

    fn validate_same_length(&self, data: &InputData) -> Result<(), IndicatorError> {
        let lengths: Array1<usize> = self.required_fields.iter().filter_map(
            |field| data.get_by_bar_field(&field).map(|arr| arr.len())
        ).collect();

        if lengths.is_empty() {
            return Err(IndicatorError::InvalidInput("Empty input.".to_string()));
        }
        let first_length = lengths[0];
        if lengths.iter().skip(1).any(|&len| len != first_length) {
            return Err(IndicatorError::InvalidInput("Input data series of the bars must have the same length.".to_string()));
        }

        Ok(())
    }

    pub fn validate_candle(&self, data: &InputData) -> Result<(), IndicatorError> {
        self.validate_required_fields_presence(data)?;
        self.validate_same_length(data)?;
        Ok(())
    }
}

impl ParameterValidator {
    fn validate_required_param(&self, params: &Value, param_name: &str) -> Result<(), IndicatorError> {
        if params.get(param_name).is_none() {
            Err(IndicatorError::InvalidParameters(
                format!("Parameter '{}' does not exist", param_name),
            ))
        } else {
            Ok(())
        }
    }

    fn validate_positive_integer_param(&self, params: &Value, param_name: &str) -> Result<(), IndicatorError> {
        if let Some(value) = params.get(param_name).and_then(|v| v.as_i64()) {
            if value > 0 {
                Ok(())
            } else {
                Err(IndicatorError::InvalidParameters(format!("Parameter '{}' must be a positive integer", param_name)))
            }
        } else {
            Err(IndicatorError::InvalidParameters(format!("Parameter '{}' must be a positive integer", param_name)))
        }
    }

    fn validate_correct_period(&self, params: &Value, left: &str, right: &str) -> Result<(), IndicatorError> {
        if let Some(left_number) = params.get(left).and_then(|v| v.as_i64()) {
            if let Some(right_number) = params.get(right).and_then(|v| v.as_i64()) {
                if left_number < right_number {
                    Ok(())
                } else {
                    Err(IndicatorError::InvalidParameters(format!("Parameter '{}' must be less than '{}'", left, right)))
                }
            } else {
                Err(IndicatorError::InvalidParameters(format!("Parameter '{}' must be a positive integer", right)))
            }
        } else {
            Err(IndicatorError::InvalidParameters(format!("Parameter '{}' must be a positive integer", left)))
        }
    }

    fn validate_less_than_data(&self, params: &Value, param_name: &str, data_len: &i64) -> Result<(), IndicatorError> {
        if let Some(value) = params.get(param_name).and_then(|v| v.as_i64()) {
            if value < *data_len {
                Ok(())
            } else {
                Err(IndicatorError::InvalidParameters(
                    format!("Parameter '{}' must be less than {}.", param_name, data_len),
                ))
            }
        } else {
            Err(IndicatorError::InvalidParameters(
                format!("Parameter '{}' must be a number.", param_name),
            ))
        }
    }

    fn validate_params(&self, params: &Value, data: &InputData) -> Result<(), IndicatorError> {
        for rule in &self.param_rules {
            match rule {
                ParamRule::Required(param_name) => self.validate_required_param(params, param_name)?,
                ParamRule::PositiveInteger(param_name) => self.validate_positive_integer_param(params, param_name)?,
                ParamRule::CorrectPeriod { left, right } => self.validate_correct_period(params, left, right)?,
                ParamRule::LessThanData {param, data_len } => self.validate_less_than_data(params, param, data_len)?,
                ParamRule::Custom(func) => {
                    func(params, data)?;
                }
            }
        }
        Ok(())
    }
}

impl Validator {
    pub fn new(required_fields: Vec<BarField>, param_rules: Vec<ParamRule>) -> Self {
        Validator {
            candle_validator: CandleValidator { required_fields },
            parameter_validator: ParameterValidator { param_rules },
        }
    }

    pub fn validate_data(&self, data: &InputData) -> Result<(), IndicatorError> {
        self.candle_validator.validate_candle(data)?;
        Ok(())
    }

    pub fn validate_params<T: IParameter + Serialize>(&self, data: &InputData, params: &T) -> Result<(), IndicatorError> {
        self.parameter_validator.validate_params(&params.to_value(), data)?;
        Ok(())
    }

    pub fn validate<T: IParameter + Serialize>(&self, data: &InputData, params: &T) -> Result<(), IndicatorError> {
        self.candle_validator.validate_candle(data)?;
        self.parameter_validator.validate_params(&params.to_value(), data)?;
        Ok(())
    }
}

pub enum ParamRule {
    Required(&'static str),
    PositiveInteger(&'static str),
    CorrectPeriod { left: &'static str, right: &'static str },
    LessThanData { param: &'static str, data_len: i64 },
    Custom(Box<dyn Fn(&Value, &InputData) -> Result<(), IndicatorError>>),
}
