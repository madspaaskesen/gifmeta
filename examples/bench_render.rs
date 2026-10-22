//! Compare single-pass rendering with rendering every frame on its own.
//! Run: `cargo run --release --example bench_render -- path/to/file.gif`
use gifmeta::{render_all_frames, utils::render::render_frame, PreviewMode};
use std::time::Instant;
fn main() {
    let path = std::env::args().nth(1).unwrap();
    let t = Instant::now();
    let all = render_all_frames(&path, PreviewMode::Composited).unwrap();
    println!(
        "render_all_frames: {} frames in {:?}",
        all.len(),
        t.elapsed()
    );
    let t = Instant::now();
    let _ = render_frame(&path, all.len() - 1, PreviewMode::Composited).unwrap();
    println!("render_frame(last): {:?}", t.elapsed());
    let t = Instant::now();
    let m = gifmeta::get_metadata(&path.clone().into(), true).unwrap();
    println!("get_metadata: {:?} ({} frames)", t.elapsed(), m.frame_count);
    let t = Instant::now();
    for i in 0..all.len() {
        let _ = render_frame(&path, i, PreviewMode::Composited).unwrap();
    }
    println!("render_frame for every frame (old way): {:?}", t.elapsed());
}
