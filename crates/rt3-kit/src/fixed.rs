//! Fixed-record sparse collections used by companies and players.
//!
//! Unlike `sparse`, these records take their length from the header and have
//! no per-record length prefix. The original mask allocation is
//! `(capacity + 8) >> 3` (`RT3.exe 0x00517d90`), including at exact multiples
//! of eight. Mask semantics remain open; the serialized record count is the
//! explicit active count in the header.

#[derive(Debug, Clone, PartialEq)]
pub struct FixedBlobRecord {
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct FixedBlobCollection {
    pub section_offset: usize,
    pub capacity: usize,
    pub records: Vec<FixedBlobRecord>,
    /// Offset of the first collection-ending tag.
    pub end_offset: usize,
    /// Raw bytes between the two ending tags when the collection carries a
    /// trailer; empty otherwise. Semantics are not inferred.
    pub trailer: Vec<u8>,
}

#[derive(Clone, Copy)]
pub(crate) struct Spec {
    pub outer: u32,
    pub inner: u32,
    pub record_bytes: usize,
    /// Record size written in the header; differs from the stored size for
    /// track lines, whose reader argument is 36 but whose header says 74.
    pub header_record_bytes: u32,
    pub nominal_slots: u32,
    /// A fixed header flag, or `None` when the company header repeats active.
    pub header_flag: Option<u32>,
    pub max_capacity: usize,
    /// Number of ending tags: one, or two with an optional raw trailer.
    pub end_tags: usize,
    pub trailer: bool,
    pub end: [u32; 2],
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

fn parse_candidate(bytes: &[u8], section: usize, spec: Spec) -> Option<FixedBlobCollection> {
    if u32_at(bytes, section)? != spec.outer
        || u32_at(bytes, section + 4)? != spec.inner
        || u32_at(bytes, section + 8)? != 1
        || u32_at(bytes, section + 12)? != spec.header_record_bytes
        || u32_at(bytes, section + 16)? != spec.nominal_slots
    {
        return None;
    }
    let capacity = u32_at(bytes, section + 24)? as usize;
    let active = u32_at(bytes, section + 28)? as usize;
    if capacity > spec.max_capacity || active > capacity {
        return None;
    }
    let header_flag = u32_at(bytes, section + 32)?;
    if match spec.header_flag {
        Some(expected) => header_flag != expected,
        None => header_flag != active as u32,
    } {
        return None;
    }

    let level0 = section.checked_add(36)?;
    if u32_at(bytes, level0)? != 1
        || u32_at(bytes, level0 + 4)? != 0
        || u32_at(bytes, level0 + 8)? != 0
    {
        return None;
    }
    let level1 = u32_at(bytes, section + 48)?;
    if level1 > 1 || u32_at(bytes, section + 60)? != 0 {
        return None;
    }
    // The fourth level triple is retained by the original but is not a
    // boolean flag in all shipped maps; no semantics are inferred from it.
    let mut off = section.checked_add(84)?;
    if level1 != 0 {
        let level_bytes = capacity.checked_mul(12)?;
        take(bytes, &mut off, level_bytes)?;
    }
    let mask_bytes = capacity.checked_add(8)? >> 3;
    take(bytes, &mut off, mask_bytes)?;

    let mut records = Vec::new();
    records.try_reserve_exact(active).ok()?;
    for _ in 0..active {
        let raw = take(bytes, &mut off, spec.record_bytes)?;
        let mut data = Vec::new();
        data.try_reserve_exact(spec.record_bytes).ok()?;
        data.extend_from_slice(raw);
        records.push(FixedBlobRecord { data });
    }
    let end_offset = off;
    if u32_at(bytes, off)? != spec.end[0] {
        return None;
    }
    let mut trailer = Vec::new();
    if spec.end_tags != 1 {
        if spec.trailer {
            let start = off + 4;
            let window = bytes.get(start..)?;
            let limit = window.len().min(64 * 1024);
            let end_tag = spec.end[1].to_le_bytes();
            let pos = window[..limit].windows(4).position(|w| w == end_tag)?;
            trailer.extend_from_slice(&window[..pos]);
        } else if u32_at(bytes, off + 4)? != spec.end[1] {
            return None;
        }
    }
    Some(FixedBlobCollection {
        section_offset: section,
        capacity,
        records,
        end_offset,
        trailer,
    })
}

pub(crate) fn read(bytes: &[u8], spec: Spec) -> Option<FixedBlobCollection> {
    let mut off = 0x3000usize.min(bytes.len());
    while off <= bytes.len().saturating_sub(4) {
        if u32_at(bytes, off) == Some(spec.outer) {
            if let Some(collection) = parse_candidate(bytes, off, spec) {
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

    fn push_u32(out: &mut Vec<u8>, value: u32) {
        out.extend_from_slice(&value.to_le_bytes());
    }

    #[test]
    fn reads_level_payload_exact_multiple_mask_and_fixed_records() {
        let spec = Spec {
            outer: 0x32d0,
            inner: 0x5209,
            record_bytes: 3243,
            header_record_bytes: 3243,
            nominal_slots: 4,
            header_flag: Some(0),
            max_capacity: 64,
            end_tags: 2,
            trailer: false,
            end: [0x520a, 0x520b],
        };
        let mut bytes = vec![0u8; 0x3000];
        for word in [spec.outer, spec.inner, 1, 3243, 4, 2, 8, 2, 0] {
            push_u32(&mut bytes, word);
        }
        for level in 0..4 {
            push_u32(&mut bytes, if level <= 1 { 1 } else { 0 });
            push_u32(&mut bytes, 0);
            push_u32(&mut bytes, 0);
        }
        bytes.extend(0..96); // capacity 8 × 12-byte level-1 entries
        bytes.extend([0xf1, 0xff]); // exact-eight capacity allocates two bytes
        bytes.extend((0..2).flat_map(|slot| vec![slot; 3243]));
        push_u32(&mut bytes, 0x520a);
        push_u32(&mut bytes, 0x520b);

        let collection = read(&bytes, spec).expect("fixed collection");
        assert_eq!(collection.section_offset, 0x3000);
        assert_eq!(collection.capacity, 8);
        assert_eq!(collection.records.len(), 2);
        assert!(collection.records[0].data.iter().all(|&byte| byte == 0));
        assert!(collection.records[1].data.iter().all(|&byte| byte == 1));
        assert_eq!(collection.end_offset + 8, bytes.len());
        for end in 0..bytes.len() {
            assert!(read(&bytes[..end], spec).is_none(), "accepted prefix {end}");
        }
    }
}
