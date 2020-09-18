#![no_std]

#[derive(Debug)]
pub struct Demon {
    /// The last sample observed by the demon
    pub last_sample: u32,

    /// The last sample used by the demon for mixing
    pub last_mixing_sample: u32,

    /// The number of consecutive samples with the same value
    pub run: u32,

    /// The current key-in-progress
    pub key: u32,

    /// The current mixing value
    pub mix: u32,

    /// The number of operations remaining to obtain a good key
    pub ops_remaining: u32,

    /// The number of samples remaining before a timeout will be raise
    pub samples_remaining: u32,
}

#[derive(Debug)]
pub enum Error {
    NeedMoreSamples,
    Timeout,
}

pub type Result<T> = core::result::Result<T, Error>;

impl Default for Demon {
    fn default() -> Self {
        Demon {
            key: 0xACACACAC,
            last_sample: 0,
            last_mixing_sample: 0,
            mix: 0xF0F0F0F0,
            run: 0,
            ops_remaining: 100,
            samples_remaining: 100_000,
        }
    }
}

impl Demon {
    pub fn take_sample(&mut self, sample: u32) -> Result<[u8; 4]> {
        // println!("{:X?} - {:X?}", self, sample);

        self.samples_remaining = self.samples_remaining.saturating_sub(1);

        // If we received the same value as the last sample, increase
        // the run streak
        if sample == self.last_sample {
            self.run = self.run.saturating_add(1);
            return Err(Error::NeedMoreSamples);
        }

        self.last_sample = sample;

        if self.run != 0 {
            // println!("mix_run");
            self.mix = self.mix.saturating_add(1 << (self.run & 31));
            self.mix = self.mix.rotate_right(self.run);
        }

        // Reset the run streak to zero, as we have new data
        self.run = 0;

        // Compare the
        let sample_bits = sample & 0b11;
        let candidate_bits = (self.last_mixing_sample & 0b11) ^ sample_bits;

        let changed_data = candidate_bits != 0;
        let new_mix = self.mix.rotate_right(candidate_bits) ^ sample_bits;

        match (changed_data, (new_mix != 0)) {
            (true, true) => {
                self.key = self.key.wrapping_add(new_mix);
                self.key ^= self.mix;
                self.key = self.key.rotate_left((candidate_bits << 2) | sample_bits);

                // Store the current sample as the last one used for mixing
                self.last_mixing_sample = sample;
                self.mix = new_mix;

                // Increment the number of change operations
                self.ops_remaining = self.ops_remaining.saturating_sub(1);
            }
            (true, false) => {
                // println!("rol");
                // Data has changed, but the new mix would be zero. Rotate Left a bit
                self.key = self.key.rotate_left(7);
            }
            (false, true) => {
                // println!("ror");
                // Data hasn't changed, but the new mix would be non-zero. Rotate Right a bit
                self.key = self.key.rotate_right(7);
            }
            (false, false) => {
                // println!("invert");
                // Everything is terrible. Invert the key
                self.key = !self.key;
            }
        }


        if self.ops_remaining == 0 {
            // reset counters to gather more entropy before returning next key
            self.ops_remaining = 100;
            self.samples_remaining = 100_000;
            Ok(self.key.to_ne_bytes())
        } else if self.samples_remaining == 0 {
            Err(Error::Timeout)
        } else {
            Err(Error::NeedMoreSamples)
        }
    }
}
