use nih_plug::prelude::*;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool};


#[derive(Params)]
pub struct BleachInjectorParams {
    /// The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
    /// these IDs remain constant, you can rename and reorder these fields as you wish. The
    /// parameters are exposed to the host in the same order they were defined. In this case, this
    /// gain parameter is stored as linear gain while the values are displayed in decibels.
    #[id = "threshold"]
    pub threshold: FloatParam,
    pub threshold_value_changed: Arc<AtomicBool>,
}


impl Default for BleachInjectorParams {
    fn default() -> Self {
        Self {
            threshold: FloatParam::new(
                "Threshold",
                1.0,
                FloatRange::Linear {
                    min: 0.5,
                    max: 1.0,
                },
            ).with_smoother(SmoothingStyle::Logarithmic(50.0))
                .with_value_to_string(Arc::new(|value| {
                format!("{:.2}", value)
            })),

            threshold_value_changed: Arc::new(AtomicBool::new(false)),
        }
    }
}