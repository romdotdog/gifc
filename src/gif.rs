use fontdue::Metrics;
use gif::Decoder;
use gifski::{progress::NoProgress, Repeat, Settings};
use rgb::RGBA8;
use std::{
    borrow::BorrowMut,
    cmp::{max, min},
    io::{Read, Write},
    thread,
    time::Instant,
};

pub fn caption<R, W>(reader: R, writer: W, mut caption: String)
where
    R: Read + Send + 'static,
    W: Write,
{
    let font: &[u8] = include_bytes!("../RobotoCondensed-Bold.ttf");
    let font = fontdue::Font::from_bytes(
        font,
        fontdue::FontSettings {
            collection_index: 0,
            scale: 48f32,
        },
    )
    .unwrap();

    let v: Vec<(Metrics, Vec<RGBA8>)> = caption
        .drain(..)
        .map(|c| {
            let (metrics, mut v) = font.rasterize(c, 48f32);
            (
                metrics,
                v.drain(..)
                    .map(|p| RGBA8 {
                        r: 255 - p,
                        g: 255 - p,
                        b: 255 - p,
                        a: 255,
                    })
                    .collect::<Vec<RGBA8>>(),
            )
        })
        .collect();

    let (text_width, text_height) = v.iter().fold((0, 0), |acc, c| {
        (
            acc.0 + c.0.width + max(c.0.xmin, 0) as usize,
            if c.0.height > acc.1 {
                c.0.height
            } else {
                acc.1
            },
        )
    });

    let mut decoder = Decoder::new(reader).unwrap();
    let width: u32 = decoder.width() as u32;
    let height: u32 = decoder.height() as u32 + 128u32;

    let canvas: Vec<RGBA8> = vec![
        RGBA8 {
            r: 255,
            g: 255,
            b: 255,
            a: 255
        };
        (width * height) as usize
    ];

    let mut canvas = imgref::Img::new(canvas, width as usize, height as usize);

    let (mut collector, gif_writer) = gifski::new(Settings {
        width: Some(width),
        height: Some(height),
        quality: 100,
        fast: false,
        repeat: Repeat::Infinite,
    })
    .unwrap();

    let padding = (width as i32 - text_width as i32) / 2i32;
    for (i, row) in canvas.rows_mut().skip(48).take(80).enumerate() {
        let mut x = padding;
        for (m, p) in &v {
            let w = m.width as i32;
            let i = i as i32 - text_height as i32 + m.height as i32 + m.ymin;
            if i >= 0 {
                let lrow = m.width * i as usize;
                if x + w > 0 && x - w < width as i32 {
                    let start = max(-x, 0);
                    let end = min(w, width as i32 - x);
                    if lrow + (end as usize) < p.len() {
                        row[(x + start) as usize..(x + end) as usize]
                            .copy_from_slice(&p[lrow + start as usize..lrow + end as usize])
                    }
                }
            }
            x += w + m.xmin;
        }
    }

    // TODO: don't clone here?
    let global = decoder.global_palette().unwrap().to_owned();

    let time = Instant::now();
    let rthread = thread::Builder::new()
        .name("decode".into())
        .spawn(move || {
            let mut index = 0;
            while let Some(frame) = decoder.read_next_frame().unwrap() {
                let t = frame.transparent;

                let c: &Vec<u8> = if let Some(ref p) = frame.palette {
                    p
                } else {
                    &global
                };

                let l = frame.left as usize;
                let w = frame.width as usize;

                let mut v = frame.buffer.iter();

                for row in canvas.rows_mut().skip(frame.top as usize + 128) {
                    for (&p, d) in v.borrow_mut().take(w).zip(row.iter_mut().skip(l)) {
                        if t.is_none() || t.unwrap() != p {
                            *d = RGBA8 {
                                r: c[p as usize * 3],
                                g: c[p as usize * 3 + 1],
                                b: c[p as usize * 3 + 2],
                                a: 255,
                            };
                        }
                    }
                }

                collector
                    .add_frame_rgba(index, canvas.clone(), index as f64 / 60f64)
                    .unwrap();

                index += 1;
            }
        })
        .unwrap();

    gif_writer.write(writer, &mut NoProgress {}).unwrap();
    rthread.join().expect("thread panick");
    eprintln!("{}ms", time.elapsed().as_millis());
}
