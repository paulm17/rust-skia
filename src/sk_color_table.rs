//! Port of `include/core/SkColorTable.h` and `src/core/SkColorTable.cpp`.

use std::sync::Arc;

/// Holds per-channel lookup tables for `SkColorFilters::Table`.
///
/// The reference stores the four tables as a 256x4 A8 `SkBitmap`. `SkBitmap` is M11, so this port
/// stores the same row-major `[alpha, red, green, blue]` bytes directly. Once `SkBitmap` exists the
/// storage can change without changing this API.
#[derive(Clone, PartialEq, Eq)]
pub struct SkColorTable {
    /// Rows in the reference's A8 image order: alpha, red, green, blue.
    f_table: [[u8; 256]; 4],
}

impl SkColorTable {
    /// `SkColorTable::Make(const uint8_t table[256])`.
    pub fn make(table: Option<&[u8; 256]>) -> Option<Arc<SkColorTable>> {
        Self::make_rgba(table, table, table, table)
    }

    /// `SkColorTable::Make(tableA, tableR, tableG, tableB)`. Null tables are the identity.
    pub fn make_rgba(
        table_a: Option<&[u8; 256]>,
        table_r: Option<&[u8; 256]>,
        table_g: Option<&[u8; 256]>,
        table_b: Option<&[u8; 256]>,
    ) -> Option<Arc<SkColorTable>> {
        if table_a.is_none() && table_r.is_none() && table_g.is_none() && table_b.is_none() {
            return None; // The table is the identity.
        }

        let identity: [u8; 256] = std::array::from_fn(|i| i as u8);
        let copy = |table: Option<&[u8; 256]>| *table.unwrap_or(&identity);

        Some(Arc::new(SkColorTable {
            f_table: [copy(table_a), copy(table_r), copy(table_g), copy(table_b)],
        }))
    }

    /// `alphaTable()`.
    pub fn alpha_table(&self) -> &[u8; 256] {
        &self.f_table[0]
    }

    /// `redTable()`.
    pub fn red_table(&self) -> &[u8; 256] {
        &self.f_table[1]
    }

    /// `greenTable()`.
    pub fn green_table(&self) -> &[u8; 256] {
        &self.f_table[2]
    }

    /// `blueTable()`.
    pub fn blue_table(&self) -> &[u8; 256] {
        &self.f_table[3]
    }

    /// `flatten`: the reference writes the four 256-byte rows through `SkWriteBuffer`.
    pub fn flatten_bytes(&self) -> [u8; 4 * 256] {
        let mut bytes = [0u8; 4 * 256];
        for (row, table) in self.f_table.iter().enumerate() {
            bytes[row * 256..(row + 1) * 256].copy_from_slice(table);
        }
        bytes
    }

    /// `Deserialize`: reads the four 256-byte rows. The `SkReadBuffer` bridge is M22; this is the
    /// underlying byte-level behavior.
    pub fn deserialize(bytes: &[u8]) -> Option<Arc<SkColorTable>> {
        if bytes.len() != 4 * 256 {
            return None;
        }
        let row = |row: usize| -> [u8; 256] {
            let mut table = [0u8; 256];
            table.copy_from_slice(&bytes[row * 256..(row + 1) * 256]);
            table
        };
        Self::make_rgba(Some(&row(0)), Some(&row(1)), Some(&row(2)), Some(&row(3)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dev_color_table_identity_and_roundtrip() {
        let identity: [u8; 256] = std::array::from_fn(|i| i as u8);
        assert!(SkColorTable::make_rgba(None, None, None, None).is_none());
        let table = SkColorTable::make_rgba(None, Some(&identity), None, None).unwrap();
        assert_eq!(table.alpha_table(), &identity);
        assert_eq!(table.red_table(), &identity);
        assert_eq!(table.blue_table(), &identity);
        let bytes = table.flatten_bytes();
        let restored = SkColorTable::deserialize(&bytes).unwrap();
        assert_eq!(restored.flatten_bytes(), bytes);
    }
}
