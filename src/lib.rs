#[macro_use]
extern crate vst;
extern crate vst_gui;

use std::f32::consts::PI;
use std::sync::{Arc, Mutex};

use vst::buffer::AudioBuffer;
use vst::editor::Editor;
use vst::plugin::{Category, Plugin, Info};

const HTML: &'static str = r#"
    <!doctype html>
    <head>
        <meta charset="utf-8">
        <meta http-equiv="x-ua-compatible" content="ie=edge">
        <title></title>
        <meta name="viewport" content="width=device-width, initial-scale=1">
        <script src="https://cdnjs.cloudflare.com/ajax/libs/p5.js/1.0.0/p5.min.js"></script>


        <style type="text/css">
        body {
            margin: 0;
        }
        </style>
    </head>

        <script>
            let draggingSlider = false
            let hoveringSlider = false
            let sliderValue = 0 // normalized, 0-1 
            let startY = 50
            let endY

            function setup() {
                createCanvas(400, 600);
                endY = height*.25
            }


            function draw() {
                // Update dragging
                if (mouseX > width/2-40 && mouseX < width/2+40) {
                    if (mouseIsPressed) {
                        sliderValue = map(mouseY, startY, endY, 0, 1, true)
                        external.invoke("setThreshold " + (1.01 - sliderValue));
                    }
                    hoveringSlider = true
                } else {
                    hoveringSlider = false
                }
                
                
                // Drawing
                background(220);
                strokeWeight(2);

                // butt
                fill('pink')
                ellipse(width / 2, height - 50, 500, 300)

                // syringe
                fill('white')
                rect(width / 2 - 50, height * .25, 100, height / 2)
                rect(width / 2 - 25, height * .75, 50, 20)
                
                // syringe top/handle
                fill(hoveringSlider ? '#eee' : 'white')
                rect(width / 2 - 40, map(sliderValue, 0, 1, startY, endY) - 20, 80, 10)
                rect(width / 2 - 30, map(sliderValue, 0, 1, startY, endY) - 10, 60, 10)
                
                // stick inside
                    fill('white')
                rect(width/2 - 20, map(sliderValue, 0, 1, startY, endY), 40, 300)

                // syringe contents
                fill('aqua')
                rect(width / 2 - 50, height * .5 + map(sliderValue, 0, 1, 0, 100), 100, height * .25 - map(sliderValue, 0, 1, 0, 100))
                
                line(width / 2 - 50, height*.25, width / 2 + 50, height*.25)
                
                // bubbles

                // stripes
                for (let i = height * 0.25; i < height * 0.75; i += 20) {
                    line(width / 2 - 50, i, width / 2 - 40, i)
                }


                line(width / 2, height * .75 + 20, width / 2, height * .75 + 100)
            }
        </script>
    </html>
"#;

struct Parameters {
    pub threshold: f32
}

fn create_javascript_callback(
    oscillator: Arc<Mutex<Parameters>>) -> vst_gui::JavascriptCallback
{
    Box::new(move |message: String| {
        let mut tokens = message.split_whitespace();

        let command = tokens.next().unwrap_or("");
        let argument = tokens.next().unwrap_or("").parse::<f32>();

        let mut locked_oscillator = oscillator.lock().unwrap();

        match command {
            "getThreshold" => {
                return locked_oscillator.threshold.to_string();
            },
            "setThreshold" => {
                if argument.is_ok() {
                    locked_oscillator.threshold = argument.unwrap();
                }
            },
            _ => {}
        }

        String::new()
    })
}

struct BleachInjector {
    sample_rate: f32,
    // We access this object both from a UI thread and from an audio processing
    // thread.
    params: Arc<Mutex<Parameters>>,
}

impl Default for BleachInjector {
    fn default() -> BleachInjector {
        let params = Arc::new(Mutex::new(
            Parameters {
                threshold: 1.0
            }
        ));

        BleachInjector {
            sample_rate: 44100.0,
            params: params.clone(),
        }
    }
}

impl Plugin for BleachInjector {
    fn get_info(&self) -> Info {
        Info {
            name: "BleachInjector".to_string(),
            vendor: "Rein van der Woerd".to_string(),
            unique_id: 25032017,

            inputs: 2,
            outputs: 2,
            parameters: 0,

            ..Info::default()
        }
    }

    fn set_sample_rate(&mut self, sample_rate: f32) {
        self.sample_rate = sample_rate as f32;
    }

    fn process(&mut self, buffer: &mut AudioBuffer<f32>) {
        let params = self.params.lock().unwrap();

        for (input, output) in buffer.zip() {
            // For each input sample and output sample in buffer
            for (in_frame, out_frame) in input.into_iter().zip(output.into_iter()) {
                let distorted;

                if *in_frame >= 0.0 {
                    distorted = in_frame.min(params.threshold) / params.threshold;
                } else {
                    distorted = in_frame.max(-params.threshold) / params.threshold;
                }

                *out_frame = distorted;
            }
        }
    }

    fn get_editor(&mut self) -> Option<Box<dyn Editor>> {
        let gui = vst_gui::new_plugin_gui(
            String::from(HTML),
            create_javascript_callback(self.params.clone()),
            Some((400, 600)));
        Some(Box::new(gui))
    }
}

plugin_main!(BleachInjector);
