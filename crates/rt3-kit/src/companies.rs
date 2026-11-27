//! Company collections (`0x32ce` / `0x61a9`) from shipped maps and saves.
//!
//! Original loader: `RT3.exe 0x00429b90` (`gpdCompanyDB`). Fixed records are
//! 30,340 bytes; the raw name field begins at record offset 4. A little-endian
//! founding/creation year is at offset 67. Other gameplay fields and mask
//! semantics remain open.

use crate::fixed::{FixedBlobCollection, FixedBlobRecord, Spec};

pub type CompanyRecord = FixedBlobRecord;
pub type CompanyCollection = FixedBlobCollection;
pub const COMPANY_BLOB_BYTES: usize = 30_340;
pub const COMPANY_NAME_OFFSET: usize = 4;
pub const COMPANY_NAME_BYTES: usize = 64;
pub const COMPANY_FOUNDED_YEAR_OFFSET: usize = 67;

pub fn name_bytes(record: &CompanyRecord) -> Option<&[u8]> {
    record
        .data
        .get(COMPANY_NAME_OFFSET..COMPANY_NAME_OFFSET + COMPANY_NAME_BYTES)
}

pub fn name(record: &CompanyRecord) -> Option<&str> {
    let used = name_bytes(record)?.split(|byte| *byte == 0).next()?;
    if used.is_empty() || !used.iter().all(|byte| (32..127).contains(byte)) {
        return None;
    }
    std::str::from_utf8(used).ok()
}

pub fn founded_year(record: &CompanyRecord) -> Option<u32> {
    let end = COMPANY_FOUNDED_YEAR_OFFSET.checked_add(4)?;
    Some(u32::from_le_bytes(
        record
            .data
            .get(COMPANY_FOUNDED_YEAR_OFFSET..end)?
            .try_into()
            .ok()?,
    ))
}

pub fn read(bytes: &[u8]) -> Option<CompanyCollection> {
    crate::fixed::read(
        bytes,
        Spec {
            outer: 0x32ce,
            inner: 0x61a9,
            record_bytes: COMPANY_BLOB_BYTES,
            header_record_bytes: COMPANY_BLOB_BYTES as u32,
            nominal_slots: 5,
            header_flag: None,
            max_capacity: 64,
            end_tags: 2,
            trailer: false,
            end: [0x61aa, 0x61ab],
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::fs;

    #[test]
    fn reads_all_shipped_company_collections() {
        let Ok(home) = std::env::var("RT3_HOME") else {
            eprintln!("RT3_HOME unset; skipping install-backed company check");
            return;
        };
        let mut paths = Vec::new();
        for dir in ["maps", "Data/Tutorial"] {
            paths.extend(
                fs::read_dir(std::path::Path::new(&home).join(dir))
                    .unwrap()
                    .filter_map(|entry| entry.ok().map(|entry| entry.path()))
                    .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("gmp")),
            );
        }
        paths.sort();
        assert_eq!(paths.len(), 43);
        let mut current = 0;
        let mut total = 0;
        let mut years = BTreeMap::new();
        for path in paths {
            let bytes = fs::read(&path).unwrap();
            current += 1;
            let collection = read(&bytes).expect("company collection");
            total += collection.records.len();
            for record in collection.records {
                assert_eq!(record.data.len(), COMPANY_BLOB_BYTES);
                assert!(name(&record).is_some());
                let name = name_bytes(&record).unwrap();
                let used = name.split(|byte| *byte == 0).next().unwrap();
                assert!(!used.is_empty());
                assert!(used.iter().all(|byte| (32..127).contains(byte)));
                *years.entry(founded_year(&record).unwrap()).or_insert(0) += 1;
            }
        }
        assert_eq!(current, 43);
        assert_eq!(total, 37);
        assert_eq!(
            years,
            BTreeMap::from([
                (1830, 19),
                (1832, 1),
                (1867, 3),
                (1870, 8),
                (1871, 1),
                (1872, 1),
                (1939, 2),
                (1964, 2),
            ])
        );
    }
}
