use maxwell::{Demon, Result as MResult, Error};
use plotters::prelude::*;

fn generate_from_loop(input: &[u32], bytes: usize, ttl_ops: u32) -> MResult<Vec<u8>> {
    let mut demon = Demon::default();
    let mut out = Vec::new();
    let mut data = input.iter().cycle();
    let mut last = [0, 0, 0, 0];
    assert_eq!(bytes % 4, 0);

    'outer: for _ in 0..(bytes / 4) {

        demon.ops_remaining = ttl_ops;
        demon.samples_remaining = ttl_ops * 1000;

        loop {
            match demon.take_sample(*data.next().unwrap()) {
                Ok(by_out) => {
                    if by_out == last {
                        println!("warning, duplicate! {:02X?}", last);
                    }
                    last = by_out;

                    out.extend_from_slice(&by_out);
                    continue 'outer;
                }
                Err(Error::NeedMoreSamples) => {},
                Err(e) => return Err(e),
            }
        }
    }
    Ok(out)
}

static TESTS: &[(&str, &[u32])] = &[
    ("one-bit-loop.png", &[0, 1]),
    ("two-bit-loop.png", &[0, 1, 2, 3]),
    ("reasonable-loop.png", &[2, 1, 0, 0, 0, 0, 1, 1, 2, 3, 1, 1, 1, 1, 0]),
    ("one-bit-skip.png", &[0, 0, 1, 1]),
    ("two-bit-skip.png", &[0, 0, 1, 1, 2, 2, 3, 3]),
    ("fib-100.png", &[1, 1, 2, 3, 5, 8, 13, 21, 34, 55, 89]),
];

fn main() -> Result<(), Box<dyn std::error::Error>> {

    let bitter = |byte: &u8| {
        let mut byte: u8 = *byte;
        let mut out = Vec::new();
        for _ in 0..8 {
            out.push((byte & 0x80) == 0x80);
            byte <<= 1;
        }
        out.into_iter()
    };

    for cts in &[10, 100, 1000, 10000] {
        for (name, data) in TESTS.iter().map(|(name, data)| (format!("{}-", cts) + name, data)) {
            let mut ones = 0;
            let mut zeros = 0;

            print!("{} - ", name);
            let root_drawing_area =
                BitMapBackend::new(&name, (1200, 1200)).into_drawing_area();
            // And we can split the drawing area into 3x3 grid
            let child_drawing_areas = root_drawing_area.split_evenly((600, 600));
            // Then we fill the drawing area with different color

            let val = generate_from_loop(data, (600 * 600) / 8, *cts).unwrap();
            let val_iter = val.iter().map(bitter).flatten();

            for (area, color) in child_drawing_areas.into_iter().zip(val_iter) {
                if color {
                    ones += 1;
                    area.fill(&RGBColor(255u8, 255u8, 255u8))?;
                } else {
                    zeros += 1;
                    area.fill(&RGBColor(0, 0, 0))?;
                }
            }

            println!("0: {}, 1: {}", zeros, ones);
        }
    }



    Ok(())
}
