use core::fmt;

#[binrw::binrw]
#[brw(little)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct ProgramId(u64);

impl fmt::Debug for ProgramId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ProgramId({:016x})", &self.0)
    }
}

impl fmt::Display for ProgramId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:016x}", &self.0)
    }
}
