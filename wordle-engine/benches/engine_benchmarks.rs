use criterion::{black_box, criterion_group, criterion_main, Criterion};
use wordle_engine::{calculate_statuses, is_word_in_list, get_adversarial_step, get_ai_word_list};

fn bench_calculate_statuses(c: &mut Criterion) {
    c.bench_function("calculate_statuses", |b| {
        b.iter(|| {
            calculate_statuses(black_box("apple"), black_box("maple"));
        })
    });
}

fn bench_is_word_in_list(c: &mut Criterion) {
    c.bench_function("is_word_in_list", |b| {
        b.iter(|| {
            is_word_in_list(black_box("apple"));
        })
    });
}

fn bench_adversarial_step(c: &mut Criterion) {
    let pool = get_ai_word_list();
    c.bench_function("adversarial_step", |b| {
        b.iter(|| {
            get_adversarial_step(black_box("slate"), black_box(pool.clone()));
        })
    });
}

criterion_group!(benches, bench_calculate_statuses, bench_is_word_in_list, bench_adversarial_step);
criterion_main!(benches);
