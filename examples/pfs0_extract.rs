/// Reads a NSP as a PFS0 and extracts all files inside
use swonch::{prelude::*, storage::FileStorage};

use std::{fs, io, path::PathBuf};

fn main() {
    let mut args = std::env::args().skip(1);

    let fpath = args.next().expect("needs path to a nsp as first argument");

    let out_dir = args
        .next()
        .map(PathBuf::from)
        .expect("needs path to extract outdir as second argument");

    let pfs0 = FileStorage::open(&fpath)
        .expect("failed to open file")
        .map::<Pfs0<_>>(())
        .expect("couldnt parse pfs0");

    println!("Extracting PFS0 from {fpath:?}");
    fs::create_dir_all(&out_dir).ok();
    for file in pfs0.files() {
        let filepath = out_dir.join(file.name().to_string());
        let mut out_fp = fs::File::create(&filepath).unwrap();
        let mut in_fp = file.data().to_file_like();

        print!("  {filepath:?}...");
        io::copy(&mut in_fp, &mut out_fp).unwrap();
        println!("Done.");
    }
}