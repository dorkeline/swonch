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

fn main() -> SwonchResult<()> {
    let mut tkey = [0; 0x10];
    let fpath = std::env::args()
        .nth(1)
        .expect("needs path to a nsp as first argument");

    FileStorage::open(fpath)?
        .map_to_storage::<Pfs0>(())
        .unwrap()
        .files()
        .find(|e| e.name().ends_with(b".tik"))
        .expect("no ticket found")
        .data()
        .unwrap()
        .read_at(0x180, &mut tkey)?;

    println!("{}", hex_str(&tkey));

    Ok(())
}
