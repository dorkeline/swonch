use bstr::BStr;

use crate::common::RightsId;

#[binrw::binrw]
#[brw(little)]
pub struct Ticket {
    #[brw(align_after = 0x40)]
    signature: Signature,
    data: TicketData,
}

#[binrw::binrw]
#[brw(little)]
pub enum Signature {
    #[brw(magic = 0x010000u32)] RSA_4096_PKCS_SHA1([u8; 0x200]),
    #[brw(magic = 0x010001u32)] RSA_2048_PKCS1_SHA1([u8; 0x100]),
    #[brw(magic = 0x010002u32)] ECDSA_SHA1([u8; 0x3c]),
    #[brw(magic = 0x010003u32)] RSA_4096_PKCS1_SHA256([u8; 0x200]),
    #[brw(magic = 0x010004u32)] RSA_2048_PKCS1_SHA256([u8; 0x100]),
    #[brw(magic = 0x010005u32)] HMAC_SHA1_160([u8; 0x14]),  
}

#[binrw::binrw]
#[brw(little)]
#[derive(Debug, Clone)]
pub struct TicketData {
    issuer: [u8; 0x40],
    title_key_block: [u8; 0x100],
    key_type: TitleKeyType,
    version: u8,
    license_type: u8,
    master_key_revision: u8,
    properties: u16,
    reserved: u64,
    ticket_id: u64,
    device_id: u64,
    rights_id: RightsId,
    account_id: u32,
    _unk0: [u8; 0xc],
    _unk1: [u8; 0x140]
}

impl TicketData {
    pub fn issuer(&self) -> &BStr {
        BStr::new(&self.issuer)
    }

    pub fn title_key(&self) -> [u8; 0x10] {
        match self.key_type {
            TitleKeyType::Common => {
                self.title_key_block[..0x10].try_into().unwrap()
            },
            TitleKeyType::Personalised => {
                todo!()
            }
        }
    }
}


#[binrw::binrw]
#[brw(repr = u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TitleKeyType {
    Common = 0,
    Personalised = 1,
}
