use core::{fmt, num::ParseIntError};

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

impl TryFrom<&str> for RightsId {
    type Error = ParseIntError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        u128::from_str_radix(value, 16).map(Self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rights_id_fmt() {
        let id = RightsId(0xcafebabedeadbeef);
        let id_s = RightsId::try_from("cafebabedeadbeef").unwrap();
        assert_eq!(id, id_s);
        assert_eq!(id.to_string(), id_s.to_string())
    }
}
