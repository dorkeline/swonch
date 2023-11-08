use criterion::{black_box, criterion_group, criterion_main, Criterion};
use std::io::{self, Read, Seek, Write};
use swonch::storage::{FileStorage, IStorage, SubStorage};
use tempfile::tempfile;

const KIB: usize = 1024;
const MIB: usize = 1024 * KIB;
const GIB: usize = 1024 * MIB;

fn std_file_write_1GiB() -> std::fs::File {
    let mut fp = tempfile().unwrap();

    let buf = vec![0; MIB];

    for i in 0..1024 {
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

fn file_storage_read_1GiB(fp: &std::fs::File) {
    let mut fp = fp.try_clone().unwrap();
    fp.seek(io::SeekFrom::Start(0)).unwrap();
    let storage = FileStorage::new(fp);

    let mut buf = vec![0; GIB];
    for i in (0..1024).step_by(16) {
        black_box(storage.read_at(i * MIB as u64, &mut buf[i as usize * MIB..][..16 * MIB]))
            .unwrap();
        black_box(&buf);
    }
}

fn file_storage_write_1GiB() {
    let fp = tempfile().map(swonch::storage::FileStorage::new).unwrap();
    let buf = vec![0; MIB];

    for i in 0..1024 {
        black_box(fp.write_at(i * MIB as u64, &buf)).unwrap();
        black_box(&buf);
    }

    black_box(buf);
}

fn substorage_file_storage_write_1GIB() {
    let fp = tempfile().unwrap();
    let fp = swonch::storage::FileStorage::new(fp);

    let buf = vec![0; MIB];

    for i in 0..1024 {
        // the tempfile doesnt seem to have a size even after flush
        let fp = SubStorage::split_from_ignore_parent_len(fp.clone(), i * MIB as u64, MIB as u64);

        black_box(fp.write_at(0, &buf)).unwrap();
        black_box(&buf);
    }

    black_box(buf);
}

fn substorage_file_storage_read_1GIB(fp: &std::fs::File) {
    let mut fp = fp.try_clone().unwrap();
    fp.seek(io::SeekFrom::Start(0)).unwrap();
    let storage = FileStorage::new(fp);

    let mut buf = vec![0; MIB];
    for i in 0..1024 {
        let sub = storage.clone().split(i * MIB as u64, MIB as u64).unwrap();
        black_box(sub.read_at(0, &mut buf)).unwrap();
        black_box(&sub);
        black_box(&buf);
    }

    black_box(buf);
}

fn criterion_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("file_storage_perfs");
    group.sample_size(20);

    group
        .bench_function("SubStorage<FileStorage> every MiB write 1GiB", |b| {
            b.iter(substorage_file_storage_write_1GIB)
        })
        .bench_function("SubStorage<FileStorage> every MiB read 1GiB", |b| {
            let fp = std_file_write_1GiB();
            b.iter(|| substorage_file_storage_read_1GIB(&fp))
        })
        /*.bench_function("std::fs::File write 1GiB", |b| b.iter(std_file_write_1GiB))
        .bench_function("std::fs::File read 1GiB", |b| {
            let fp = std_file_write_1GiB();
            b.iter(|| std_file_read_1GiB(&fp))
        })
        .bench_function("FileStorage read 1GiB", |b| {
            let fp = std_file_write_1GiB();
            b.iter(|| file_storage_read_1GiB(&fp))
        })
        .bench_function("FileStorage write 1GiB", |b| {
            b.iter(file_storage_write_1GiB)
        })*/;
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
