use core::fmt;

#[binrw::binrw]
#[brw(big)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RightsId(pub u128);

impl fmt::Debug for RightsId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ProgramId({:032x})", &self.0)
    }
}

impl fmt::Display for RightsId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "0x{:032x}", &self.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rights_id_fmt() {
        let id = RightsId(0xcafebabedeadbeef);
        assert_eq!(id.to_string(), "0xcafebabedeadbeef")
    }
}
