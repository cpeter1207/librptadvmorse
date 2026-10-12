use super::MorseRenderer;

fn render_all(renderer: &mut MorseRenderer, block: usize) -> Vec<f32> {
    let mut rendered = Vec::new();
    loop {
        let mut output = vec![99.0; block];
        let count = renderer.render(&mut output);
        rendered.extend_from_slice(&output[..count]);
        if count == 0 {
            return rendered;
        }
    }
}

#[test]
fn morse_preserves_literal_48khz_timing_and_validation() {
    for (text, speed, samples) in [
        ("", 20, 0),
        ("E", 20, 2_880),
        ("T", 20, 8_640),
        ("ET", 20, 20_160),
        ("E T", 20, 31_680),
        ("I", 20, 8_640),
        ("EE", 17, 16_941),
    ] {
        let mut renderer = MorseRenderer::new(text, speed, 1_000.0, -6).unwrap();
        assert_eq!(render_all(&mut renderer, 4_096).len(), samples);
    }
    for (text, speed, frequency, level) in [
        ("E*", 20, 1_000.0, -6),
        ("E", 0, 1_000.0, -6),
        ("E", 101, 1_000.0, -6),
        ("E", 20, 0.0, -6),
        ("E", 20, 24_000.0, -6),
        ("E", 20, f32::NAN, -6),
        ("E", 20, f32::INFINITY, -6),
        ("E", 20, 1_000.0, -61),
    ] {
        assert!(
            MorseRenderer::new(text, speed, frequency, level).is_err(),
            "{text} {speed} {frequency} {level}"
        );
    }
}

#[test]
fn morse_is_partition_invariant_and_resets_keyed_phase() {
    let mut whole = MorseRenderer::new("E T I", 20, 1_000.0, -6).unwrap();
    let expected = render_all(&mut whole, 65_536);
    let mut blocks = MorseRenderer::new("E T I", 20, 1_000.0, -6).unwrap();
    assert_eq!(blocks.render(&mut []), 0);
    assert_eq!(render_all(&mut blocks, 37), expected);
    assert!(expected[12] > 0.5);
    assert!(expected[36] < -0.5);
    assert!(expected[2_880..23_040].iter().all(|sample| *sample == 0.0));
}

#[test]
fn morse_accepts_the_complete_supported_alphabet_and_has_no_trailing_gap() {
    let mut renderer = MorseRenderer::new(
        "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789/.,?-=+@()'!\":;_$&",
        100,
        1_000.0,
        -6,
    )
    .unwrap();
    assert!(!render_all(&mut renderer, 257).is_empty());
    let mut e = MorseRenderer::new("E", 20, 1_000.0, -6).unwrap();
    let samples = render_all(&mut e, 4_096);
    assert_eq!(samples.len(), 2_880);
    assert_ne!(samples[samples.len() - 1], 0.0);
}

#[test]
fn restart_replays_the_message_from_its_initial_phase() {
    let mut renderer = MorseRenderer::new("E", 20, 1_000.0, -6).unwrap();
    let mut first = [0.0; 64];
    assert_eq!(renderer.render(&mut first), first.len());
    renderer.restart();
    let mut replay = [0.0; 64];
    assert_eq!(renderer.render(&mut replay), replay.len());
    assert_eq!(replay, first);
}

#[test]
fn shared_morse_matches_baseline_bits() {
    let alphabet = "ABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789/.,?-=+@()'!\":;_$&";
    // Captured independently from unchanged controller source on native runners:
    // https://github.com/cpeter1207/librptadvmorse/actions/runs/38287693814
    // The two architectures happen to agree; each has its own original-source anchor.
    let expected_hashes = match std::env::consts::ARCH {
        "x86_64" => [
            0x8b939bdaf931e465_u64,
            0x68ab9113b68e4c03,
            0x1d783fb1fa366fc0,
        ],
        "aarch64" => [
            0x8b939bdaf931e465_u64,
            0x68ab9113b68e4c03,
            0x1d783fb1fa366fc0,
        ],
        architecture => panic!("missing original-source baseline for {architecture}"),
    };
    for ((text, speed, frequency, level, expected_len), expected_hash) in [
        (alphabet, 100, 1_000.0, -6, 461_376),
        ("EE E T I", 17, 1_000.0, -6, 111_811),
        ("E T I E", 37, 1_250.0, -11, 45_145),
    ]
    .into_iter()
    .zip(expected_hashes)
    {
        for block in [1, 7, 960] {
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
            assert_eq!(count, expected_len, "block {block}");
            assert_eq!(hash, expected_hash, "block {block}");
        }
    }
    let mut renderer = MorseRenderer::new("E T I E", 37, 1_250.0, -11).unwrap();
    let mut first = [0.0; 960];
    assert_eq!(renderer.render(&mut first), first.len());
    renderer.restart();
    let mut replay = [0.0; 960];
    assert_eq!(renderer.render(&mut replay), replay.len());
    assert!(
        first
            .iter()
            .zip(replay)
            .all(|(a, b)| a.to_bits() == b.to_bits())
    );
    assert_eq!(first[0].to_bits(), 0);
    assert_eq!(first[1].to_bits(), 0x3d3c0c51);
}
