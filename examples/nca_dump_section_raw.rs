use swonch::{keyset::KEYS, prelude::*, storage::FileStorage};

use std::{fs, io::Write, path::PathBuf, time::Instant};

fn main() -> SwonchResult<()> {
    env_logger::init();
    let _ = KEYS.init_from_default_locations();

    let mut args = std::env::args().skip(1);
    let fpath = args.next().expect("needs path to a nca as first argument");

    let out_dir = args
        .next()
        .map(PathBuf::from)
        .expect("arg2 needs to be a path to an outdir");
    fs::create_dir_all(&out_dir).ok();

    let nca = FileStorage::open(&fpath)?.map_to_storage::<Nca>(())?;

    for section in nca.sections() {
        let dec = section.open_decrypted()?;

        print!(
            "extracting section {} [{}]...",
            section.index(),
            humansize::format_size(dec.length()?, humansize::BINARY)
        );
        let t0 = Instant::now();

        let mut fp =
            fs::File::create(out_dir.join(format!("section{}_dec", section.index()))).unwrap();

        /*
            we could use io::copy(&mut dec.into_stdio(), &mut fp) instead for the ergonomics
            but copying manually with a decently sized buffer is very, very fast, speeds it up by a factor of ~10.
            on my machine its about 2-3 times faster than hactool
         */

        let mut buf = vec![0; 16 * 1024 * 1024];
        let mut off = 0;
        loop {
            let read = dec.read_at(off, &mut buf)?;
            off += read;
            fp.write_all(&buf[..read as usize])?;
            if read < buf.len() as u64 {
                break;
            }
        }

        println!("Done after {:?}.", Instant::now() - t0);
    }

    Ok(())
}
