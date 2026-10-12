#![allow(dead_code)]

mod audio {
    pub trait AudioSource {
        fn render(&mut self, output: &mut [f32]) -> usize;
    }
    #[path = "/workspace/.work/rpt_advanced/rust/core/src/audio/morse.rs"]
    pub mod morse;
}

use audio::morse::MorseRenderer;

fn bits(text: &str, speed: u32, frequency: f32, level: i8, block: usize) -> (usize, u64) {
    let mut renderer = MorseRenderer::new(text, speed, frequency, level).unwrap();
    let mut count = 0;
    let mut hash = 0xcbf29ce484222325_u64;
    loop {
        let mut output = vec![99.0; block];
        let written = renderer.render(&mut output);
        if written == 0 {
            break;
        }
        count += written;
        for sample in &output[..written] {
            for byte in sample.to_bits().to_le_bytes() {
                hash ^= u64::from(byte);
                hash = hash.wrapping_mul(0x100000001b3);
            }
        }
    }
    (count, hash)
}

fn main() {
    let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789/.,?-=+@()'!\":;_$&";
    for (name, text, speed, frequency, level) in [
        ("alphabet", alphabet, 100, 1_000.0, -6),
        ("fractional", "EE E T I", 17, 1_000.0, -6),
        ("phase", "E T I E", 37, 1_250.0, -11),
    ] {
        for block in [1, 7, 960] {
            let (count, hash) = bits(text, speed, frequency, level, block);
            println!("{name} {block} {count} {hash:016x}");
        }
    }
    let mut renderer = MorseRenderer::new("E T I E", 37, 1_250.0, -11).unwrap();
    let mut first = [0.0; 960];
    renderer.render(&mut first);
    renderer.restart();
    let mut replay = [0.0; 960];
    renderer.render(&mut replay);
    println!(
        "restart {}",
        first
            .iter()
            .zip(replay)
            .all(|(a, b)| a.to_bits() == b.to_bits())
    );
    println!(
        "element_bits {:08x} {:08x}",
        first[0].to_bits(),
        first[1].to_bits()
    );
}
