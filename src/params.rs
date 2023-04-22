use nih_plug::prelude::*;

use std::sync::Arc;


#[derive(Params)]
pub struct BleachInjectorParams {
    /// The parameter's ID is used to identify the parameter in the wrappred plugin API. As long as
    /// these IDs remain constant, you can rename and reorder these fields as you wish. The
    /// parameters are exposed to the host in the same order they were defined. In this case, this
    /// gain parameter is stored as linear gain while the values are displayed in decibels.
    #[id = "threshold"]
    pub threshold: FloatParam,
}


impl Default for BleachInjectorParams {
    fn default() -> Self {
        Self {
            threshold: FloatParam::new(
                "Threshold",
                1.0,
                FloatRange::Skewed {
                    min: 0.00,
                    max: 0.999,
                    factor: 3.0,
                },
            ).with_smoother(SmoothingStyle::Linear(50.0))
                .with_value_to_string(Arc::new(|value| {
                format!("{:.2}", value)
            })),
        }
    }
}