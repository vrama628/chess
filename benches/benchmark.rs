use chess::{ai, game::Game};
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark(c: &mut Criterion) {
    let game = Game::new();
    c.bench_function("ai", |b| b.iter(|| ai::choose(&game, 4)));
}

criterion_group!(group, benchmark);
criterion_main!(group);
