mod params;


extern crate math;
extern crate nih_plug;
extern crate nih_plug_webview;
extern crate serde;
extern crate serde_json;
extern crate open;

use nih_plug::prelude::*;
use nih_plug_webview::*;
use serde::Deserialize;
use serde_json::json;
use std::sync::atomic::{Ordering};

use params::BleachInjectorParams;


use std::sync::{Arc};


#[derive(Deserialize)]
#[serde(tag = "type")]
pub enum Action {
    Init,
    SetSize { width: u32, height: u32 },
    SetThreshold { value: f32 },
    OpenWebsite
}

struct BleachInjector {
    params: Arc<BleachInjectorParams>,
}


impl Default for BleachInjector {
    fn default() -> Self {
        Self {
            params: Arc::new(BleachInjectorParams::default()),
        }
    }
}

impl Plugin for BleachInjector {
    const NAME: &'static str = "Bleach Injector";
    const VENDOR: &'static str = "SoftPack6ix";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "reinvanderwoerd@me.com";

    const VERSION: &'static str = env!("CARGO_PKG_VERSION");

    // The first audio IO layout is used as the default. The other layouts may be selected either
    // explicitly or automatically by the host or the user depending on the plugin API/backend.
    const AUDIO_IO_LAYOUTS: &'static [AudioIOLayout] = &[AudioIOLayout {
        main_input_channels: NonZeroU32::new(2),
        main_output_channels: NonZeroU32::new(2),

        aux_input_ports: &[],
        aux_output_ports: &[],

        // Individual ports and the layout as a whole can be named here. By default these names
        // are generated as needed. This layout will be called 'Stereo', while a layout with
        // only one input and output channel would be called 'Mono'.
        names: PortNames::const_default(),
    }];


    const MIDI_INPUT: MidiConfig = MidiConfig::None;
    const MIDI_OUTPUT: MidiConfig = MidiConfig::None;

    const SAMPLE_ACCURATE_AUTOMATION: bool = true;

    // If the plugin can send or receive SysEx messages, it can define a type to wrap around those
    // messages here. The type implements the `SysExMessage` trait, which allows conversion to and
    // from plain byte buffers.
    type SysExMessage = ();
    // More advanced plugins can use this to run expensive background tasks. See the field's
    // documentation for more information. `()` means that the plugin does not have any background
    // tasks.
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn editor(&self, _async_executor: AsyncExecutor<Self>) -> Option<Box<dyn Editor>> {
        let params = self.params.clone();
        let editor = WebViewEditor::new(HTMLSource::String(include_str!("gui.html")), (400, 600))
            .with_background_color((150, 150, 150, 255))
            .with_developer_mode(true)
            .with_event_loop(move |ctx, setter| {
                while let Some(event) = ctx.next_event() {
                    match event {
                        WebviewEvent::JSON(value) => {
                            if let Ok(action) = serde_json::from_value(value) {
                                match action {
                                    Action::SetThreshold { value } => {
                                        setter.begin_set_parameter(&params.threshold);
                                        setter.set_parameter_normalized(&params.threshold, value);
                                        setter.end_set_parameter(&params.threshold);
                                    }
                                    Action::SetSize { width, height } => {
                                        ctx.resize(width, height);
                                    }
                                    Action::Init => {
                                        let _ = ctx.send_json(json!({
                                            "type": "set_size",
                                            "width": ctx.width.load(Ordering::Relaxed),
                                            "height": ctx.height.load(Ordering::Relaxed)
                                        }));
                                    }
                                    Action::OpenWebsite => {
                                        open::that("https://klokpacksix.nl").unwrap();
                                    }
                                }
                            } else {
                                panic!("Invalid action received from web UI.")
                            }
                        }
                        WebviewEvent::FileDropped(path) => println!("File dropped: {:?}", path),
                        WebviewEvent::ParamValueChanged(param, value) => {
                            match &param as &str {
                                "threshold" => {
                                    let _ = ctx.send_json(json!({
                                        "type": "set_threshold",
                                        "value": value
                                    }));
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
            });

        Some(Box::new(editor))
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        // Resize buffers and perform other potentially expensive initialization operations here.
        // The `reset()` function is always called right after this function. You can remove this
        // function if you do not need it.
        true
    }

    fn reset(&mut self) {
        // Reset buffers and envelopes here. This can be called from the audio thread and may not
        // allocate. You can remove this function if you do not need it.
    }


    fn process(
        &mut self,
        buffer: &mut Buffer,
        _aux: &mut AuxiliaryBuffers,
        _context: &mut impl ProcessContext<Self>,
    ) -> ProcessStatus {
        for channel_samples in buffer.iter_samples() {
            // Smoothing is optionally built into the parameters themselves
            let gain = self.params.threshold.smoothed.next();

            for sample in channel_samples {
                *sample *= gain;
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for BleachInjector {
    const CLAP_ID: &'static str = "com.sokpack6ix.bleach-injector";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A short description of your plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;

    // Don't forget to change these features
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::AudioEffect, ClapFeature::Stereo];
}

impl Vst3Plugin for BleachInjector {
    const VST3_CLASS_ID: [u8; 16] = *b"Exactly16Chars!!";

    // And also don't forget to change these categories
    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Dynamics];
}

nih_export_clap!(BleachInjector);
nih_export_vst3!(BleachInjector);