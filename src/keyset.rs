use crate::common::RightsId;
use crate::sync_impl::RwLock;

use alloc::collections::BTreeMap;

lazy_static::lazy_static! {
    pub static ref KEYS: Keyset = Keyset::empty();
}

pub struct Keyset {
    prod: RwLock<BTreeMap<String, Key>>,
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

    #[error("error parsing key")]
    Parsing(#[from] crate::utils::ParseKeyError),
}

#[derive(Clone)]
enum Key {
    Single(Vec<u8>),
    Versioned(BTreeMap<u8, Vec<u8>>),
}

pub trait FromRawKey: Sized {
    fn from_key(key: &[u8]) -> Result<Self, KeyError>;
}

/// A pair of Aes128 keys for XTS(N) mode, one for the block crypto the other for the tweak. 
#[derive(Debug, Clone)]
pub struct Aes128XtsKey(pub [u8; 0x20]);

impl Aes128XtsKey {
    pub fn crypt(&self) -> aes::Aes128 {
        use aes::{Aes128, cipher::{KeyInit, generic_array::GenericArray}};

        Aes128::new(GenericArray::from_slice(&self.0[..0x10]))
    }

    pub fn tweak(&self) -> aes::Aes128 {
        use aes::{Aes128, cipher::{KeyInit, generic_array::GenericArray}};

        Aes128::new(GenericArray::from_slice(&self.0[0x10..]))
    }
}

impl FromRawKey for Aes128XtsKey {
    fn from_key(key: &[u8]) -> Result<Self, KeyError> {
        key.try_into().map(Self).map_err(|e| {
            crate::utils::ParseKeyError::LengthMismatch {
                requested_key_len: 0x20,
                actual_key_len: key.len(),
            }
            .into()
        })
    }
}

impl Keyset {
    pub fn empty() -> Self {
        Self {
            prod: RwLock::new(BTreeMap::new()),
            titles: RwLock::new(BTreeMap::new()),
        }
    }

    pub fn get_key<K: FromRawKey>(&self, key_name: impl AsRef<str>) -> Result<K, KeyError> {
        let key_name = key_name.as_ref();
        match self.get_key_impl(key_name)? {
            Key::Single(key) => K::from_key(&key),
            Key::Versioned(keys) => {
                // if theres only one key and its version 0 make it work without an index
                if keys.keys().len() == 1 {
                    if let Some(key) = keys.get(&0) {
                        return K::from_key(key);
                    }
                }

                Err(KeyError::RequestedStandaloneForVersionedKey {
                    key_name: key_name.into(),
                })
            }
        }
    }

    pub fn get_key_index<K: FromRawKey>(
        &self,
        key_name: impl AsRef<str>,
        key_index: u8,
    ) -> Result<K, KeyError> {
        let key_name = key_name.as_ref();
        match self.get_key_impl(key_name)? {
            Key::Single(key) => {
                // make index 0 work for single keys too, remove if it causes issues
                if key_index == 0 {
                    return K::from_key(&key);
                }
                Err(KeyError::RequestedIndexForStandaloneKey {
                    key_name: key_name.into(),
                })
            }
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
        self.prod
            .read()
            .get(key_name)
            .ok_or(KeyError::MissingKey {
                key_name: key_name.into(),
            })
            .cloned()
    }

    pub fn insert_key(
        &self,
        key_name: impl AsRef<str>,
        key: impl Into<Vec<u8>>,
        index: Option<u8>,
    ) {
        let mut keys = self.prod.write();
        let name = key_name.as_ref();
        let key = key.into();

        if let Some(existing_key) = keys.get_mut(name) {
            match (existing_key, index) {
                (Key::Single(ref mut prev), None) => {
                    log::trace!("replaced key {name} = {prev:?} with {key:?}");
                    let _ = core::mem::replace(prev, key);
                }
                (Key::Single(_), Some(_)) => {
                    log::warn!("tried to insert an indexed key when an unversioned one with the same name {name:?} already existed");
                }
                (Key::Versioned(map), Some(i)) => {
                    map.insert(i, key);
                }
                (Key::Versioned(_), None) => {
                    log::warn!("tried to insert an unversioned key when a versioned one with the same name {name:?} already existed");
                }
            }
        } else {
            let key = match index {
                Some(idx) => {
                    let mut map = BTreeMap::new();
                    map.insert(idx, key);
                    Key::Versioned(map)
                }
                None => Key::Single(key),
            };

            keys.insert(name.into(), key);
        }
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
