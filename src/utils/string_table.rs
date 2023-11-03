//! Null terminated string table

use alloc::vec::Vec;
use bstr::{BStr, ByteSlice};
use core::{fmt, iter::FusedIterator};

/// A string table containing strings that are seperated
/// by null and can be accessed either by index or iteration.
///
/// # Note
/// The string table assumes every string is null terminated **including the last one**.
/// It won't cause memory safety issues but it will just not give you the last string.
///
/// This table uses the [`bstr::BStr`] type to more conveniently work with
/// utf8-ish data and not just ascii.
#[derive(Clone)]
#[binrw::binrw]
#[br(import(size: usize))]
pub struct StringTable(#[br(count = size)] Vec<u8>);

impl StringTable {
    /// Creates a StringTable from a raw bytes backend. If you have a iterator of strings
    /// consider using the `FromIterator` implementation instead by `.collect()`'ing.
    ///
    /// # Example
    /// ```
    /// use swonch::utils::string_table::StringTable;
    ///
    /// let raw: Vec<u8> = b"foo\0bar\0".into();
    /// let st = StringTable::from_raw(raw);
    ///
    /// assert!(st.get(0).is_some());
    /// ```
    pub fn from_raw(raw: impl Into<Vec<u8>>) -> Self {
        Self(raw.into())
    }

    /// Gets a string from the string table by byte index of the first character, **not** the nth' string.
    /// If you want to get the nth string consider using `table.iter().nth(n)` instead.
    ///
    /// # Example
    /// ```
    /// use swonch::utils::string_table::StringTable;
    /// use bstr::ByteSlice;
    ///
    /// let st: StringTable = ["foo", "bar"].iter().collect();
    ///
    /// assert_eq!(st.get(0).unwrap(), b"foo".as_bstr());
    /// assert_eq!(st.get(4).unwrap(), b"bar".as_bstr());
    /// ```
    pub fn get(&self, index: usize) -> Option<&BStr> {
        self.0.get(index..).and_then(|start| {
            start
                .iter()
                .position(|c| *c == 0)
                .map(|end| start[..end].as_bstr())
        })
    }

    /// Iterates over all strings in the string table.
    ///
    /// # Example
    /// ```
    /// use swonch::utils::string_table::StringTable;
    ///
    /// let st: StringTable = ["foo", "bar"].iter().collect();
    ///
    /// for string in st.iter() {
    ///     println!("{string}");
    /// }
    /// ```
    pub fn iter(&self) -> impl Iterator<Item = &BStr> {
        StringTableIter {
            table: self,
            offset: 0,
        }
    }

    /// Retrieves the underlying byte array.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl<S: AsRef<[u8]>> FromIterator<S> for StringTable {
    fn from_iter<T: IntoIterator<Item = S>>(iter: T) -> Self {
        let mut inner = Vec::new();

        for string in iter {
            let s = string.as_ref();

            // only slice up to the first 0 byte
            let slice = &s[..s.find([0]).unwrap_or(s.len())];

            if let Some(b) = slice.last() {
                inner.extend_from_slice(slice);
                if *b != 0 {
                    inner.push(0);
                }
            }
        }

        Self(inner)
    }
}

impl fmt::Debug for StringTable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}

struct StringTableIter<'a> {
    table: &'a StringTable,
    offset: usize,
}

impl<'a> Iterator for StringTableIter<'a> {
    type Item = &'a BStr;

    fn next(&mut self) -> Option<Self::Item> {
        self.table.get(self.offset).map(|s| {
            self.offset += s.len() + 1;
            s
        })
    }
}

impl<'a> FusedIterator for StringTableIter<'a> {}

#[cfg(test)]
mod tests {
    use super::StringTable;

    #[test]
    fn from_iterator_strips_null_properly() {
        let strings: &[&[u8]] = &[*&b"foo", *&b"foo\0", *&b"foo\0\0", *&b"foo\0foo\0\0foo"];
        let table: StringTable = strings.iter().collect();

        for string in table.iter() {
            let s: &[u8] = string.as_ref();
            assert_eq!(s, *&b"foo");
        }

        assert_eq!(table.as_bytes().len(), strings.len() * b"foo\0".len())
    }
}
