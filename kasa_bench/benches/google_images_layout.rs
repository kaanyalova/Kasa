use criterion::{black_box, criterion_group, criterion_main, Criterion};
use kasa_core::{db::schema::Image, layout::google_photos::calculate_layout};
use rand::Rng;
/*
fn criterion_benchmark(c: &mut Criterion) {
    //c.sample_size(50);
    let mut images = vec![];
    let width = 1000;

    for _ in 0..=1_000_000 {
        let x = rand::thread_rng().gen_range(0..2000);
        let y = rand::thread_rng().gen_range(0..2000);

        let data = Image {
            hash: "".to_string(),
            resolution_x: x,
            resolution_y: y,
        };

        images.push(data);
    }

    c.bench_function("layout", |b| {
        b.iter(|| calculate_layout(black_box(images.clone()), 1000, 200))
    });
}
criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
*/
