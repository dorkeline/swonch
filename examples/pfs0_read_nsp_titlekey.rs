/// Reads a NSP as a PFS0, looks for a .tik file inside
/// and reads the titlekey from a hardcoded position
use swonch::{prelude::*, storage::FileStorage};

fn hex_str(b: &[u8]) -> String {
    use core::fmt::Write;

    let mut s = String::with_capacity(b.len() * 2);
    for byte in b {
        write!(s, "{byte:02x}").ok();
    }
    s
}

fn main() {
    let mut buf = [0; 0x10];
    let fpath = std::env::args()
        .nth(1)
        .expect("needs path to a nsp as first argument");

    FileStorage::open(fpath)
        .expect("failed to open file")
        .map::<Pfs0<_>>(())
        .expect("couldnt parse pfs0")
        .files()
        .find(|e| e.name().ends_with(b".tik"))
        .expect("no ticket found")
        .data()
        .read_at(0x180, &mut buf)
        .expect("couldnt read from nested file in pfs0");

    println!("tkey: {}", hex_str(&buf));
}
