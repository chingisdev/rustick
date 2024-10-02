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

pub fn calculate_true_range(
    high: &Array1<f64>,
    low: &Array1<f64>,
    close: &Array1<f64>,
) -> Result<Array1<f64>, IndicatorError> {
    let length = high.len();
    let mut tr = Array1::<f64>::zeros(length);

    for i in 0..length {
        if i == 0 {
            tr[i] = high[i] - low[i];
        } else {
            let hl = high[i] - low[i];
            let hc = (high[i] - close[i - 1]).abs();
            let lc = (low[i] - close[i - 1]).abs();
            tr[i] = hl.max(hc).max(lc);
        }
    }

    Ok(tr)
}


fn get_signed_directional_movement(
    target: &Array1<f64>,
    comparative: &Array1<f64>,
) -> Array1<f64> {
    target.iter()
        .zip(comparative.iter())
        .map(|(&target, &comparative)| if target > comparative && target > 0.0 {target} else {0.0})
        .collect::<Array1<f64>>()
}

pub fn calculate_directional_movements(
    high: &Array1<f64>,
    low: &Array1<f64>,
) -> Result<(Array1<f64>, Array1<f64>), IndicatorError> {
    let up_move = &high.slice(s![1..]) - &high.slice(s![..high.len() - 1]);
    let down_move = &low.slice(s![..low.len() - 1]) - &low.slice(s![1..]);

    let zero_array = Array1::<f64>::zeros(up_move.len());

    let plus_dm = get_signed_directional_movement(&up_move, &down_move);

    let minus_dm = get_signed_directional_movement(&down_move, &up_move);

    // Prepend a zero to align the output array lengths with the input
    let mut plus_dm_full = Array1::<f64>::zeros(high.len());
    plus_dm_full.slice_mut(s![1..]).assign(&plus_dm);

    let mut minus_dm_full = Array1::<f64>::zeros(low.len());
    minus_dm_full.slice_mut(s![1..]).assign(&minus_dm);

    Ok((plus_dm_full, minus_dm_full))
}


pub fn wilder_smoothing(
    data: &Array1<f64>,
    period: usize,
) -> Result<Array1<f64>, IndicatorError> {
    let length = data.len();
    if length < period {
        return Err(IndicatorError::InvalidInput(
            "Data length must be at least equal to the period for smoothing".to_string(),
        ));
    }

    let mut smoothed = Array1::<f64>::zeros(length);

    // First smoothed value is the average of the first 'period' data points
    let initial_average = data.slice(s![..period]).mean().unwrap();
    smoothed[period - 1] = initial_average;

    // Compute smoothed values for the rest of the data
    for i in period..length {
        smoothed[i] = smoothed[i - 1] + (data[i] - smoothed[i - 1]) / period as f64;
    }

    Ok(smoothed)
}


#[cfg(test)]
mod tests {
    use ndarray::{array, s, Array1};
    use super::wilder_smoothing;

    #[test]
    fn test_wilder_smoothing() {
        let data = array![2.0, 3.0, 5.0, 6.0, 9.0, 11.0, 13.0];
        let period = 3;

        let smoothed = wilder_smoothing(&data, period).unwrap();

        // Expected smoothed values calculated manually or from a trusted source
        let expected = array![
        0.0, 0.0,  // First two values are zeros since smoothing starts at index period - 1
        3.333333,  // Average of first three data points: (2 + 3 + 5) / 3
        4.222222,  // 3.333333 + (6 - 3.333333) / 3
        5.814815,  // 4.222222 + (9 - 4.222222) / 3
        7.54321,  // 5.814815 + (11 - 5.814815) / 3
        9.362139   // 7.54321 + (13 - 7.54321) / 3
    ];

        // Adjust for the initial zeros
        let mut adjusted_expected = Array1::<f64>::zeros(data.len());
        adjusted_expected.slice_mut(s![period - 1..]).assign(&expected.slice(s![period - 1..]));

        for i in 0..data.len() {
            println!("index:{}, smothed={}, expected={}", i, smoothed[i], adjusted_expected[i]);
            if i < period - 1 {
                assert_eq!(smoothed[i], 0.0);
            } else {
                assert!(
                    (smoothed[i] - adjusted_expected[i]).abs() < 0.0001,
                    "Smoothed value at index {} does not match expected value",
                    i
                );
            }
        }
    }

}
