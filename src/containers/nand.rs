pub struct Nand {
    _boot0: (),
    _boot1: (),
    _gpt: (),
}

pub enum NandPartition {
    System,
    User,
    Safe,
}

impl Nand {}
