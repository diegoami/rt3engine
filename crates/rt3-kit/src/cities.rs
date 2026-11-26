//! City collections (`0x332d` / `0x5209`) from `.gmp` maps.
//!
//! Original reader: `RT3.exe 0x00421510` (`gpdCityDB`). This module
//! proves structural framing and retains raw labels; gameplay meanings
//! of the remaining blob fields are open.

pub const CITY_BLOB_BYTES: usize = 905;
pub const LABEL_OFFSET: usize = 854;
pub const LABEL_BYTES: usize = 48;
const MAX_CITY_SLOTS: usize = 1024;

#[derive(Debug, Clone, PartialEq)]
pub struct CityRecord {
    pub slot: usize,
    pub prefix: [u8; 6],
    pub data: Vec<u8>,
}

impl CityRecord {
    /// Raw fixed-size label field. Some shipped records have leading
    /// non-text bytes; no silent normalization is performed.
    pub fn label_bytes(&self) -> Option<&[u8]> {
        self.data.get(LABEL_OFFSET..LABEL_OFFSET + LABEL_BYTES)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct CityCollection {
    pub section_offset: usize,
    pub capacity: usize,
    pub records: Vec<CityRecord>,
    /// Offset of the `0x520a` tag ending the fixed collection area.
    pub end_offset: usize,
}

fn u32_at(bytes: &[u8], off: usize) -> Option<u32> {
    let end = off.checked_add(4)?;
    Some(u32::from_le_bytes(bytes.get(off..end)?.try_into().ok()?))
}

fn take<'a>(bytes: &'a [u8], off: &mut usize, len: usize) -> Option<&'a [u8]> {
    let end = off.checked_add(len)?;
    let out = bytes.get(*off..end)?;
    *off = end;
    Some(out)
}

fn parse_candidate(bytes: &[u8], section: usize) -> Option<CityCollection> {
    let collection = section.checked_add(4)?;
    let header = section.checked_add(8)?;
    if u32_at(bytes, section)? != 0x332d
        || u32_at(bytes, collection)? != 0x5209
        || u32_at(bytes, header)? != 0
        || u32_at(bytes, header + 4)? != 6
        || u32_at(bytes, header + 8)? != 10
        || u32_at(bytes, header + 12)? != 20
        || u32_at(bytes, header + 24)? != 0
    {
        return None;
    }
    let capacity = u32_at(bytes, header + 16)? as usize;
    let active = u32_at(bytes, header + 20)? as usize;
    if capacity > MAX_CITY_SLOTS || active > capacity {
        return None;
    }
    for i in 0..4 {
        let level = header.checked_add(28 + i * 12)?;
        let expected = if i == 0 { 1 } else { 0 };
        if u32_at(bytes, level)? != expected
            || u32_at(bytes, level + 4)? != 0
            || u32_at(bytes, level + 8)? != 0
        {
            return None;
        }
    }
    let mut off = header.checked_add(76)?;
    let mask_bytes = capacity.checked_add(7)? / 8;
    let mask = take(bytes, &mut off, mask_bytes)?;
    let mut records = Vec::new();
    records.try_reserve_exact(active).ok()?;
    for slot in 0..capacity {
        if mask[slot / 8] & (1 << (slot % 8)) != 0 {
            continue;
        }
        if records.len() >= active {
            return None;
        }
        let prefix = take(bytes, &mut off, 6)?.try_into().ok()?;
        let len = u16::from_le_bytes(take(bytes, &mut off, 2)?.try_into().ok()?) as usize;
        if len != CITY_BLOB_BYTES {
            return None;
        }
        let raw = take(bytes, &mut off, CITY_BLOB_BYTES)?;
        let mut data = Vec::new();
        data.try_reserve_exact(CITY_BLOB_BYTES).ok()?;
        data.extend_from_slice(raw);
        records.push(CityRecord { slot, prefix, data });
    }
    if records.len() != active || u32_at(bytes, off)? != 0x520a {
        return None;
    }
    Some(CityCollection {
        section_offset: section,
        capacity,
        records,
        end_offset: off,
    })
}

pub fn read(bytes: &[u8]) -> Option<CityCollection> {
    let mut off = 0x3000usize.min(bytes.len());
    while off <= bytes.len().saturating_sub(4) {
        if u32_at(bytes, off) == Some(0x332d) {
            if let Some(collection) = parse_candidate(bytes, off) {
                return Some(collection);
            }
        }
        off += 1;
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn push_u32(out: &mut Vec<u8>, value: u32) {
        out.extend_from_slice(&value.to_le_bytes());
    }

    fn blob(label: &[u8]) -> Vec<u8> {
        let mut data = vec![0u8; CITY_BLOB_BYTES];
        data[LABEL_OFFSET..LABEL_OFFSET + label.len()].copy_from_slice(label);
        data
    }

    #[test]
    fn reads_sparse_city_slots_and_raw_labels() {
        let mut bytes = vec![0u8; 0x3000];
        push_u32(&mut bytes, 0x332d);
        push_u32(&mut bytes, 0x5209);
        for word in [0, 6, 10, 20, 3, 2, 0] {
            push_u32(&mut bytes, word);
        }
        for level in 0..4 {
            push_u32(&mut bytes, if level == 0 { 1 } else { 0 });
            push_u32(&mut bytes, 0);
            push_u32(&mut bytes, 0);
        }
        bytes.push(0b010); // slot 1 absent; slots 0 and 2 active
        for (prefix, label) in [(11u8, b"Dublin".as_slice()), (22, b"Cork".as_slice())] {
            bytes.extend_from_slice(&[prefix; 6]);
            bytes.extend_from_slice(&(CITY_BLOB_BYTES as u16).to_le_bytes());
            bytes.extend_from_slice(&blob(label));
        }
        push_u32(&mut bytes, 0x520a);
        let collection = read(&bytes).expect("city collection");
        assert_eq!(collection.capacity, 3);
        assert_eq!(collection.records.len(), 2);
        assert_eq!(collection.records[0].slot, 0);
        assert_eq!(collection.records[1].slot, 2);
        assert!(collection.records[0]
            .label_bytes()
            .unwrap()
            .starts_with(b"Dublin"));
        assert!(collection.records[1]
            .label_bytes()
            .unwrap()
            .starts_with(b"Cork"));
        for end in 0..bytes.len() {
            assert!(read(&bytes[..end]).is_none(), "accepted city prefix {end}");
        }
        let mut bad_active = bytes.clone();
        bad_active[0x301c..0x3020].copy_from_slice(&3u32.to_le_bytes());
        assert!(read(&bad_active).is_none());
        let mut bad_length = bytes.clone();
        let first_length = 0x3000 + 8 + 76 + 1 + 6;
        bad_length[first_length..first_length + 2]
            .copy_from_slice(&((CITY_BLOB_BYTES as u16 + 1).to_le_bytes()));
        assert!(read(&bad_length).is_none());
        bytes[collection.end_offset..collection.end_offset + 4]
            .copy_from_slice(&0u32.to_le_bytes());
        assert!(read(&bytes).is_none());
    }

    #[test]
    fn reads_all_shipped_city_collections() {
        let Ok(home) = std::env::var("RT3_HOME") else {
            eprintln!("RT3_HOME unset; skipping install-backed city check");
            return;
        };
        let mut paths = Vec::new();
        for dir in ["maps", "Data/Tutorial"] {
            paths.extend(
                fs::read_dir(std::path::Path::new(&home).join(dir))
                    .unwrap()
                    .filter_map(|e| e.ok().map(|e| e.path()))
                    .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("gmp")),
            );
        }
        paths.sort();
        assert_eq!(paths.len(), 43);
        let mut total = 0;
        let mut control_prefixed = 0;
        let mut dublin = false;
        let mut manchester = false;
        for path in paths {
            let bytes = fs::read(&path).unwrap();
            let collection =
                read(&bytes).unwrap_or_else(|| panic!("no city collection in {}", path.display()));
            assert!(collection
                .records
                .iter()
                .all(|r| r.data.len() == CITY_BLOB_BYTES));
            total += collection.records.len();
            control_prefixed += collection
                .records
                .iter()
                .filter(|r| r.label_bytes().unwrap()[0] != 0 && r.label_bytes().unwrap()[0] < 32)
                .count();
            let name = path.file_stem().unwrap().to_string_lossy();
            if name == "Ireland" {
                dublin = collection
                    .records
                    .iter()
                    .any(|r| r.label_bytes().unwrap().starts_with(b"Dublin"));
            }
            if name == "British Isles" {
                manchester = collection
                    .records
                    .iter()
                    .any(|r| r.label_bytes().unwrap().starts_with(b"Manchester"));
            }
        }
        assert_eq!(total, 3_726);
        assert_eq!(control_prefixed, 3);
        assert!(dublin && manchester);
    }
}
