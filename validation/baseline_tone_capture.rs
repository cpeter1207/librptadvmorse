#![allow(dead_code)]

mod audio {
    pub trait AudioSource {
        fn render(&mut self, output: &mut [f32]) -> usize;
    }
    #[path = "/workspace/.work/rpt_advanced/rust/core/src/audio/tone.rs"]
    pub mod tone;
}

use audio::AudioSource;
use audio::tone::ToneSequence;

fn main() {
    for text in ["1100/1,1100/1", "1000+1100@-6/1", "silence/1,1000/1"] {
        for block in [256, 7] {
            let mut sequence = ToneSequence::new(text, -20).unwrap();
            let mut count = 0;
            let mut hash = 0xcbf29ce484222325_u64;
            let mut last_written = 0;
            loop {
                let mut output = vec![99.0; block];
                let written = sequence.render(&mut output);
                if written == 0 {
                    break;
                }
                count += written;
                last_written = written;
                for sample in &output[..written] {
                    for byte in sample.to_bits().to_le_bytes() {
                        hash ^= u64::from(byte);
                        hash = hash.wrapping_mul(0x100000001b3);
                    }
                }
            }
            println!("tone {text} {block} {count} {last_written} {hash:016x}");
        }
    }
}
