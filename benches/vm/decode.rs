use criterion::{black_box, criterion_group, criterion_main, Criterion};
use chip_8_rs::vm::VM;

const codes: [u16; 34] = [
    0x8120,
    0xF133,
    0x8121,
    0x8122,
    0x8123,
    0x8126,
    0x812E,
    0x3122,
    0x4122,
    0x5120,
    0x9120,
    0x6122,
    0x7122,
    0x00E0,
    0xD12A,
    0x1333,
    0x00EE,
    0x2333,
    0xB333,
    0xE19E,
    0xE1A1,
    0xF10A,
    0x8124,
    0x8125,
    0x8127,
    0xA333,
    0xF11E,
    0xF129,
    0xF155,
    0xF165,
    0xC122,
    0xF118,
    0xF107,
    0xF115,
];

fn criterion_benchmark(c: &mut Criterion) {
    for code in codes {
        c.bench_function(&format!("decode {}", code), |b| b.iter(|| VM::decode(black_box(code))));
    }
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);