use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct APOParams {
    #[serde(default = "default_fast_period")]
    pub fast_period: usize,
    #[serde(default = "default_slow_period")]
    pub slow_period: usize,
}

fn default_fast_period() -> usize { 12 }
fn default_slow_period() -> usize { 26 }

