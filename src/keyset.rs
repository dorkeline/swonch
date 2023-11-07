use crate::common::RightsId;
use crate::sync_impl::RwLock;

use alloc::collections::BTreeMap;

lazy_static::lazy_static! {
    pub static ref KEYS: Keyset = Keyset::empty();
}

pub struct Keyset {
    console: (),
    titles: RwLock<BTreeMap<RightsId, TitleKey>>,
}

pub type TitleKey = [u8; 0x10];

#[derive(Debug, Clone, thiserror_no_std::Error)]
pub enum KeyError {
    #[error("no titlekey found in db for {rights_id}")]
    NoTitlekeyForRightsId { rights_id: RightsId },

    #[error("key {key_name:?} not found")]
    MissingKey { key_name: String },

    #[error("tried to index a standalone (unversioned) key ({key_name:?})")]
    RequestedIndexForStandaloneKey { key_name: String },

    #[error("couldnt find index {index} for key {key_name:?}")]
    IndexNotFoundForKey { key_name: String, index: u8 },

    #[error("tried to access a versioned key ({key_name:?}) without index")]
    RequestedStandaloneForVersionedKey { key_name: String },
}

enum Key {
    Single(Vec<u8>),
    Versioned(BTreeMap<u8, Vec<u8>>),
}

pub trait FromRawKey: Sized {
    fn from_key(key: &[u8]) -> Result<Self, KeyError>;
}

impl Keyset {
    pub fn empty() -> Self {
        Self {
            console: (),
            titles: RwLock::new(BTreeMap::new()),
        }
    }

    pub fn get_key<K: FromRawKey>(&self, key_name: impl AsRef<str>) -> Result<K, KeyError> {
        let key_name = key_name.as_ref();
        match self.get_key_impl(key_name)? {
            Key::Single(key) => K::from_key(&key),
            Key::Versioned(_) => Err(KeyError::RequestedStandaloneForVersionedKey {
                key_name: key_name.into(),
            }),
        }
    }

    pub fn get_key_index<K: FromRawKey>(
        &self,
        key_name: impl AsRef<str>,
        key_index: u8,
    ) -> Result<K, KeyError> {
        let key_name = key_name.as_ref();
        match self.get_key_impl(key_name)? {
            Key::Single(_) => Err(KeyError::RequestedIndexForStandaloneKey {
                key_name: key_name.into(),
            }),
            Key::Versioned(map) => map
                .get(&key_index)
                .ok_or(KeyError::IndexNotFoundForKey {
                    key_name: key_name.into(),
                    index: key_index,
                })
                .and_then(|key| K::from_key(&key)),
        }
    }

    fn get_key_impl(&self, key_name: &str) -> Result<Key, KeyError> {
        todo!()
    }

    pub fn insert_key(&self, key_name: impl AsRef<str>, key: &[u8], index: Option<u8>) {
        todo!()
    }

    pub fn insert_titlekey(
        &self,
        rights_id: impl Into<RightsId>,
        title_key: impl Into<TitleKey>,
    ) -> Option<TitleKey> {
        self.titles
            .write()
            .insert(rights_id.into(), title_key.into())
    }

    pub fn get_titlekey(&self, rights_id: impl Into<RightsId>) -> Result<TitleKey, KeyError> {
        let rights_id = rights_id.into();
        self.titles
            .read()
            .get(&rights_id)
            .copied()
            .ok_or(KeyError::NoTitlekeyForRightsId { rights_id })
    }
}
