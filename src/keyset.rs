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

    pub fn insert_key(
        &self,
        key_name: impl AsRef<str>,
        key: impl Into<Vec<u8>>,
        index: Option<u8>,
    ) {
        todo!()
    }

    pub fn insert_titlekeys_from_ini(&self, reader: impl crate::io::Read) {
        let mut titles = self.titles.write();
        parse_from_ini(reader, |name, key, idx| {
            if let Some(_) = idx {
                return log::error!("titlekey had index in name?? {name} {idx:?}");
            }

            let Ok(rights_id) = name.try_into() else {
                return log::error!("couldnt parse rights id from {name:?}");
            };

            let key_len = key.len();
            let Ok(title_key) = key.try_into() else {
                return log::error!("title key has the wrong size {key_len}, has to be 16");
            };

            titles.insert(rights_id, title_key);
        })
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

fn parse_from_ini(
    mut reader: impl crate::io::Read,
    mut insert_fn: impl FnMut(&str, Vec<u8>, Option<u8>),
) {
    let mut s = String::new();
    reader.read_to_string(&mut s).unwrap();

    for line in s.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with(';') {
            continue;
        }

        let mut split_line = line.split('=');

        let (Some(name), Some(key)) = (
            split_line.next().map(str::trim),
            split_line.next().map(str::trim),
        ) else {
            log::warn!("malformed line in keys ini: {line}");
            continue;
        };

        let Ok(key) = crate::utils::hex_str_to_vec(key) else {
            log::warn!("malformed key {key}");
            continue;
        };
        let (name, idx) = split_name_into_name_and_index(name);

        insert_fn(name, key, idx);
    }
}

fn split_name_into_name_and_index(name: &str) -> (&str, Option<u8>) {
    match name
        .split('_')
        .last()
        .and_then(|s| Some((s, u8::from_str_radix(s, 16).ok()?)))
    {
        Some((suf, idx)) => (&name[..name.len() - (suf.len() + 1)], Some(idx)),
        None => (name, None),
    }
}
