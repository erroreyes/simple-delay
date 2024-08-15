use nih_plug::prelude::*;
use std::sync::Arc;

mod delaybuffer;

#[derive(Enum, PartialEq)]
enum Mode {
    INTER, 
    WRONG, 
    DIGIT,
}

struct SimpleDelay {
    params: Arc<SimpleDelayParams>,
    delay_buffer: delaybuffer::DelayBuffer,
}

#[derive(Params)]
struct SimpleDelayParams {
    // #[id = "gain"]
    // pub gain: FloatParam,
    #[id = "delay"]
    pub delay: FloatParam,
    #[id = "feedback"]
    pub feedback: FloatParam,
    #[id = "pitch"]
    pub pitch: FloatParam,
    #[id = "mix"]
    pub mix: FloatParam,
    #[id = "freeze"]
    pub freeze: BoolParam,
    #[id = "mode"]
    pub mode: EnumParam<Mode>,
}

impl Default for SimpleDelay {
    fn default() -> Self {
        Self {
            params: Arc::new(SimpleDelayParams::default()),
            delay_buffer: delaybuffer::DelayBuffer::default(),
        }
    }
}

impl Default for SimpleDelayParams {
    fn default() -> Self {
        Self {
/* 
            gain: FloatParam::new(
                "Gain",
                util::db_to_gain(0.0),
                FloatRange::Skewed {
                    min: util::db_to_gain(-6.0),
                    max: util::db_to_gain(6.0),
                    factor: FloatRange::gain_skew_factor(-6.0, 6.0),
                },
            )
            .with_smoother(SmoothingStyle::Logarithmic(50.0))
            .with_unit(" dB")
            .with_value_to_string(formatters::v2s_f32_gain_to_db(2))
            .with_string_to_value(formatters::s2v_f32_gain_to_db()),
 */
            delay: FloatParam::new(
                "Delay",
                0.2,
                FloatRange::Skewed { min: 0.01, max: 2.0, factor: 0.34},
            )
            .with_smoother(SmoothingStyle::Logarithmic(20.0)),

            feedback: FloatParam::new(
                "Feedback",
                0.2, 
                FloatRange::Linear { min: 0.0, max: 1.0 }
            )
            .with_unit("%")
            .with_value_to_string(formatters::v2s_f32_percentage(0))
            .with_string_to_value(formatters::s2v_f32_percentage()),

            pitch: FloatParam::new(
                "Pitch", 
                1.0,
                FloatRange::SymmetricalSkewed { min: 1.00, max: 13.0, factor: 1.0, center: 7.0 }
            )
            .with_smoother(SmoothingStyle::Logarithmic(20.0)),


            mix: FloatParam::new(
                "Mix",
                0.5,
                FloatRange::Linear { min: 0.0, max: 1.0 }
            ),

            freeze: BoolParam::new("Freeze", false),

            mode: EnumParam::new("Mode", Mode::INTER)
        }
    }
}

impl Plugin for SimpleDelay {
    const NAME: &'static str = "Simple-Delay";
    const VENDOR: &'static str = "LASHLIGHT";
    const URL: &'static str = env!("CARGO_PKG_HOMEPAGE");
    const EMAIL: &'static str = "your@email.com";

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

    type SysExMessage = ();
    type BackgroundTask = ();

    fn params(&self) -> Arc<dyn Params> {
        self.params.clone()
    }

    fn initialize(
        &mut self,
        _audio_io_layout: &AudioIOLayout,
        _buffer_config: &BufferConfig,
        _context: &mut impl InitContext<Self>,
    ) -> bool {
        let num_channels = _audio_io_layout.main_output_channels.expect("No output channels").get() as usize;
        self.delay_buffer.resize(num_channels, _buffer_config.sample_rate);

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
        let num_samples = buffer.samples(); // 1024
        let output = buffer.as_slice();
        // let output_len = output.len(); // 2

        for smp_idx in 0..num_samples {
            let delay = self.params.delay.smoothed.next() / self.params.pitch.smoothed.next();
            
            let gain = /* self.params.gain.smoothed.next() */ 1.0;
            let mix = self.params.mix.smoothed.next();
            let feedback = self.params.feedback.smoothed.next();
    
            let sample_l = output[0][smp_idx];
            let sample_r = output[1][smp_idx];

            let (delay_output_l, delay_output_r) = match self.params.mode.value() {
                // Mode::INTER => (self.delay_buffer.read_inter(0), self.delay_buffer.read_inter(1)),
                Mode::INTER => (self.delay_buffer.read(0), self.delay_buffer.read(1)),
                Mode::WRONG => (self.delay_buffer.read_wrong(0), self.delay_buffer.read_wrong(1)),
                Mode::DIGIT => (self.delay_buffer.read_lin(0, delay), self.delay_buffer.read_lin(1, delay)),
            };

            if self.params.freeze.value() {
                output[0][smp_idx] = (mix * delay_output_l) * gain;
                output[1][smp_idx] = (mix * delay_output_r) * gain;
                self.delay_buffer.write(0, delay_output_l);
                self.delay_buffer.write(1, delay_output_r);
            } else {
                output[0][smp_idx] = ((1.0 - mix) * sample_l + mix * delay_output_l) * gain;
                output[1][smp_idx] = ((1.0 - mix) * sample_r + mix * delay_output_r) * gain;
                self.delay_buffer.write(0, sample_l + (delay_output_l * feedback));
                self.delay_buffer.write(1, sample_r + (delay_output_r * feedback));
            }

            match self.params.mode.value() {
                Mode::DIGIT => self.delay_buffer.advance_digit(delay),
                _ => self.delay_buffer.advance_to(delay),
            }
        }

        ProcessStatus::Normal
    }
}

impl ClapPlugin for SimpleDelay {
    const CLAP_ID: &'static str = "com.lashlight.simple-delay";
    const CLAP_DESCRIPTION: Option<&'static str> = Some("A simple delay plugin");
    const CLAP_MANUAL_URL: Option<&'static str> = Some(Self::URL);
    const CLAP_SUPPORT_URL: Option<&'static str> = None;
    const CLAP_FEATURES: &'static [ClapFeature] = &[ClapFeature::Delay, ClapFeature::Stereo];
}

impl Vst3Plugin for SimpleDelay {
    const VST3_CLASS_ID: [u8; 16] = *b"lashlightsmpdl01";

    const VST3_SUBCATEGORIES: &'static [Vst3SubCategory] =
        &[Vst3SubCategory::Fx, Vst3SubCategory::Delay];
}

nih_export_clap!(SimpleDelay);
nih_export_vst3!(SimpleDelay);
