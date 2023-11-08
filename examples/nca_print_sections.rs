use swonch::{keyset::KEYS, prelude::*, storage::FileStorage};

use std::{fs, io, path::PathBuf};

fn main() -> SwonchResult<()> {
    env_logger::init();
    let _ = KEYS.init_from_default_locations();

    let mut args = std::env::args().skip(1);
    let fpath = args.next().expect("needs path to a nca as first argument");

    let nca = FileStorage::open(&fpath)?.map_to_storage::<Nca>(())?;

    println!("{nca:#?}");
    for section in nca.sections() {
        let mut buf = vec![0; 1024 * 0x20];
        let dec = section.open_decrypted()?;
        dec.read_at(0, &mut buf)?;

        //let mut fp =

        //swonch::utils::dbg_hexdump(std::io::stdout(), &buf);
    }

    Ok(())
}
