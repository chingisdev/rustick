use ndarray::{s, Array1};
use crate::models::indicator::IndicatorError;

pub fn calculate_adl(
    high: &Array1<f64>,
    low: &Array1<f64>,
    close: &Array1<f64>,
    volume: &Array1<f64>,
) -> Result<Array1<f64>, IndicatorError> {
    let length = high.len();

    // Compute the high_low_range (High - Low)
    let high_low_range = high - low;

    // To avoid division by zero, create a mask where high_low_range == 0.0
    let zero_range_mask = high_low_range.mapv(|x| x == 0.0);

    // Compute Money Flow Multiplier (MFM)
    let mfm_numerator = (close - low) - (high - close);
    let mut mfm = &mfm_numerator / &high_low_range;

    // Handle division by zero by setting MFM to zero where high_low_range == 0
    mfm.iter_mut()
        .zip(zero_range_mask.iter())
        .for_each(|(mfm_value, &is_zero)| {
            if is_zero {
                *mfm_value = 0.0;
            }
        });

    // Compute Money Flow Volume (MFV)
    let mfv = &mfm * volume;

    // Compute ADL as cumulative sum of MFV
    let adl = cumulative_sum(&mfv);

    Ok(adl)
}

pub fn calculate_ema(
    data: &Array1<f64>,
    period: usize,
) -> Result<Array1<f64>, IndicatorError> {
    if period == 0 || period > data.len() {
        return Err(IndicatorError::InvalidParameters(
            "Invalid period for EMA calculation".to_string(),
        ));
    }

    let length = data.len();
    let mut ema = Array1::<f64>::zeros(length);

    let alpha = 2.0 / (period as f64 + 1.0);

    // Initialize EMA with the first data point
    ema[period - 1] = data.slice(s![..period]).mean().unwrap();

    for i in period..length {
        ema[i] = alpha * data[i] + (1.0 - alpha) * ema[i - 1];
    }

    Ok(ema)
}

pub fn cumulative_sum(input: &Array1<f64>) -> Array1<f64> {
    let mut cumsum = Array1::zeros(input.len());
    let mut sum = 0.0;
    for (i, &value) in input.iter().enumerate() {
        sum += value;
        cumsum[i] = sum;
    }
    cumsum
}

