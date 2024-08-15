#[warn(dead_code)]

const MAX_DELAY_SECONDS: usize = 2;

#[derive(Default,Debug)]
pub struct DelayBuffer {
    sample_rate: f32,
    buffer: Vec<Vec<f32>>,
    buffer_size: usize,
    rp: usize,
    wp: usize,
}

impl DelayBuffer {
    pub fn resize(&mut self, num_channels: usize, sample_rate: f32) {
        self.sample_rate = sample_rate;
        self.buffer_size = MAX_DELAY_SECONDS * self.sample_rate as usize;

        self.buffer.resize_with(num_channels, Vec::new);
        
        for buffer in self.buffer.iter_mut() {
            buffer.resize(self.buffer_size, 0.0);
        }
    }
    
    pub fn read(&mut self, ch_idx: usize) -> f32 {
        // self.audio_buffer[ch_idx][self.rp]
        // TODO: EVERYTHING BELOW THIS LINE IS NEW. TO REVERT, DELETE EVERYTHING BELOW, AND 
        //       UNCOMMENT THE LINE ABOVE.

        self.rp = (self.rp + 1) % (self.sample_rate) as usize;
        self.buffer[ch_idx][self.rp]
    }
    
    // This is the original read method
    pub fn read_wrong(&mut self, ch_idx: usize) -> f32 {
        self.buffer[ch_idx][self.rp]
    }
    
    pub fn read_inter(&mut self, ch_idx: usize) -> f32 {
        self.rp = (self.rp + 1) % (self.sample_rate) as usize;
        // let ceil = (self.rp + 1) % self.buffer_size;
        let tmp = if self.rp == 0 {
            self.rp
        } else {
            self.rp - 1
        };
        let ceil = tmp % self.buffer_size;
        
        self.buffer[ch_idx][self.rp] + 0.02 * (self.buffer[ch_idx][self.rp] - self.buffer[ch_idx][ceil])
    }

    pub fn read_lin(&mut self, ch_idx: usize, delay: f32) -> f32 {
        self.rp = (self.rp + 1) % (self.sample_rate) as usize;

        let frac_delay = (delay * self.sample_rate) % self.buffer_size as f32;
        let frac = frac_delay % 1.0;
        (1.0 - frac) * self.buffer[ch_idx][self.rp + 1] + frac * self.buffer[ch_idx][self.rp]
    }
    
    pub fn write(&mut self, ch_idx: usize,  sample: f32) {
        self.buffer[ch_idx][self.wp] = sample;
        
    }

    pub fn advance_to(&mut self, delay: f32) {
        // NOTE: this is the original pointer move that yielded a glitchy digital steppy sound
        // when changing the delay parameter.
        // This method uses the same pointer to read and write, therefore something like:
        //     self.rp = self.wp;
        // should be call **after** setting up self.wp as in the line below.
        // self.wp = (self.wp + 1) % (delay * self.sample_rate) as usize;

        // This yields a fast pitching effect when the delay parameter is changed.
        self.wp = (self.wp + 1) % (self.sample_rate) as usize;
        self.rp = (self.wp - (delay * self.sample_rate) as usize) % self.buffer_size;

        // self.rp = (self.wp + 1) % (delay * self.sample_rate) as usize;
        // self.wp = (self.wp + 1) % (self.sample_rate) as usize; ---- OG
        // self.wp = (self.wp + 1) % (self.buffer_size) as usize;
    }

    pub fn advance_digit(&mut self, delay: f32) {
        // NOTE: this is the original pointer move that yielded a glitchy digital steppy sound
        // when changing the delay parameter.
        // This method uses the same pointer to read and write, therefore something like:
        //     self.rp = self.wp;
        // should be call **after** setting up self.wp as in the line below.
        self.wp = (self.wp + 1) % (delay * self.sample_rate) as usize;
        self.rp = self.wp;
    }
}