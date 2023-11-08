#![allow(non_snake_case)]

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::io::{self, Read, Seek, Write};
use swonch::storage::{crypto::AesCtrStorage, FileStorage, IStorage};
use tempfile::tempfile;

const KIB: usize = 1024;
const MIB: usize = 1024 * KIB;
const GIB: usize = 1024 * MIB;

use aes::cipher::*;
const AES_CTR_KEY: [u8; 0x10] = [
    0xca, 0xfe, 0xba, 0xbe, 0xca, 0xfe, 0xba, 0xbe, 0xca, 0xfe, 0xba, 0xbe, 0xca, 0xfe, 0xba, 0xbe,
];
const AES_CTR_IV: [u8; 0x10] = [0; 0x10];
type Aes128Ctr = ctr::Ctr64LE<aes::Aes128>;

fn std_file_write_1GiB() -> std::fs::File {
    let mut fp = tempfile().unwrap();

    let buf = vec![0; MIB];

    for _ in 0..1024 {
        fp.write(&buf).unwrap();
    }

    black_box(buf);
    fp.flush().unwrap();
    fp
}

fn std_file_read_1GiB(mut fp: &std::fs::File) {
    let mut buf = Vec::with_capacity(GIB);

    fp.seek(io::SeekFrom::Start(0)).unwrap();
    fp.read_to_end(&mut buf).unwrap();

    black_box(buf);
}

fn fs_file_aes_write_1GIB() -> std::fs::File {
    let mut fp = tempfile().unwrap();
    let zeroes = vec![0; MIB];
    let mut buf = vec![0; MIB];

    let mut aes_ctx = Aes128Ctr::new_from_slices(&AES_CTR_KEY, &AES_CTR_IV).unwrap();

    for i in 0..1024 {
        aes_ctx.seek(i * MIB);
        aes_ctx.apply_keystream_b2b(&zeroes, &mut buf).unwrap();
        fp.write_all(&buf).unwrap();
    }

    fp   
}

fn fs_file_aes_read_1GIB(fp: &std::fs::File) {
    let mut fp = fp.try_clone().unwrap();
    let mut buf = Vec::with_capacity(GIB);

    let mut aes_ctx = Aes128Ctr::new_from_slices(&AES_CTR_KEY, &AES_CTR_IV).unwrap();

    fp.read_to_end(&mut buf).unwrap();
    aes_ctx.apply_keystream(&mut buf);

    black_box(&buf);
}

fn aes_ctr_storage_write_1GIB() {
    let fp = tempfile().map(swonch::storage::FileStorage::new).unwrap();
    let buf = vec![0; MIB];
    let aes_ctx = Aes128Ctr::new_from_slices(&AES_CTR_KEY, &AES_CTR_IV).unwrap();
    let fp = AesCtrStorage::new(fp, aes_ctx);

    for i in 0..1024 {
        black_box(fp.write_at(i * MIB as u64, &buf)).unwrap();
        black_box(&buf);
    }

    black_box(buf);
}

fn aes_ctr_storage_read_1GIB(fp: &std::fs::File) {
    let aes_ctx = Aes128Ctr::new_from_slices(&AES_CTR_KEY, &AES_CTR_IV).unwrap();
    let fp = fp.try_clone().unwrap();
    let fp = AesCtrStorage::new(FileStorage::new(fp), aes_ctx);

    let mut buf = vec![0; GIB];
    for i in (0..1024).step_by(16) {
        black_box(fp.read_at(i * MIB as u64, &mut buf[i as usize * MIB..][..16 * MIB]))
            .unwrap();
        black_box(&buf);
    }
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("aes_file_storage_perf");
    group.sample_size(20);

    group
        .bench_function("AesCtrStorage write 1GiB", |b| b.iter(aes_ctr_storage_write_1GIB))
        .bench_function("AesCtrStorage read 1GiB", |b| {
            let fp = fs_file_aes_write_1GIB();
            b.iter(|| aes_ctr_storage_read_1GIB(&fp))
        })
        .bench_function("std::fs::File AesCtr write 1GiB", |b| b.iter(fs_file_aes_write_1GIB))
        .bench_function("std::fs::File AesCtr read 1GiB", |b| {
            let fp = fs_file_aes_write_1GIB();
            b.iter(|| fs_file_aes_read_1GIB(&fp))
        })
        .bench_function("std::fs::File write 1GiB", |b| b.iter(std_file_write_1GiB))
        .bench_function("std::fs::File read 1GiB", |b| {
            let fp = std_file_write_1GiB();
            b.iter(|| std_file_read_1GiB(&fp))
        });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
