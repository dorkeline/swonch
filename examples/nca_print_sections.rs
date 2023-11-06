use swonch::{prelude::*, storage::FileStorage};

use std::{fs, io, path::PathBuf};

fn main() -> SwonchResult<()> {
    let mut args = std::env::args().skip(1);

    let fpath = args.next().expect("needs path to a nca as first argument");

    let header_key = args
        .next()
        .map(|s| swonch::utils::hex_str_to_array::<0x20>(&s).expect("couldnt parse header_key"))
        .expect("needs header_key as second argument");

    let nca = FileStorage::open(&fpath)?.map_to_storage::<Nca>(Some(header_key))?;

    Ok(())
}
