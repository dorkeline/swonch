experimenting with making a switch file formats driver in rust that easily supports nested containers transparently. 

this isnt really viable until i added a crypto layer implementation and verified with benchmarks that my design approach is not awful

### example
opening a NSP (common dump format) as a Pfs0 and reading the titlekey from the ticket
```rs
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
        .map_to_storage::<Pfs0>(())?
        .files()
        .find(|e| e.name().ends_with(b".tik"))
        .expect("no ticket found")
        .data()?
        .read_at(0x180, &mut tkey)?;

    println!("{}", hex_str(&tkey));

    Ok(())
}
```
