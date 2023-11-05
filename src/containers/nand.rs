pub struct Nand {
    boot0: (),
    boot1: (),
    gpt: (),
}

pub enum NandPartition {
    System,
    User,
    Safe,
}

impl Nand {}
