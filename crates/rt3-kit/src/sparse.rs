//! Shared sparse fixed-blob collections used by buildings and events.

pub const LABEL_TAIL_BYTES: usize = 51;
pub const LABEL_BYTES: usize = 48;
const MAX_SLOTS: usize = 1024;

#[derive(Debug, Clone, PartialEq)]
pub struct SparseBlobRecord {
    pub slot: usize,
    pub prefix: [u8; 6],
    pub data: Vec<u8>,
}

impl SparseBlobRecord {
    pub fn label_bytes(&self) -> Option<&[u8]> {
        let start = self.data.len().checked_sub(LABEL_TAIL_BYTES)?;
        self.data.get(start..start + LABEL_BYTES)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct SparseBlobCollection {
    pub section_offset: usize,
    pub capacity: usize,
    pub records: Vec<SparseBlobRecord>,
    pub end_offset: usize,
}

#[derive(Clone, Copy)]
pub(crate) struct Spec {
    pub outer: u32,
    pub inner: u32,
    pub end: u32,
    pub preamble: Option<u32>,
    pub blob_bytes: usize,
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

fn parse_candidate(bytes: &[u8], section: usize, spec: Spec) -> Option<SparseBlobCollection> {
    let inner = section.checked_add(4)?;
    let mut header = inner.checked_add(4)?;
    if u32_at(bytes, section)? != spec.outer || u32_at(bytes, inner)? != spec.inner {
        return None;
    }
    if let Some(value) = spec.preamble {
        if u32_at(bytes, header)? != value {
            return None;
        }
        header = header.checked_add(4)?;
    }
    if u32_at(bytes, header)? != 0
        || u32_at(bytes, header + 4)? != 6
        || u32_at(bytes, header + 8)? != 10
        || u32_at(bytes, header + 12)? != 20
        || u32_at(bytes, header + 24)? != 0
    {
        return None;
    }
    let capacity = u32_at(bytes, header + 16)? as usize;
    let active = u32_at(bytes, header + 20)? as usize;
    if capacity > MAX_SLOTS || active > capacity {
        return None;
    }
    let level_offset = header.checked_add(28)?;
    let mut levels = [0u32; 4];
    for (i, flag) in levels.iter_mut().enumerate() {
        let level = level_offset.checked_add(i * 12)?;
        *flag = u32_at(bytes, level)?;
        if (i == 0
            && (*flag != 1 || u32_at(bytes, level + 4)? != 0 || u32_at(bytes, level + 8)? != 0))
            || (i > 0 && *flag > 1)
        {
            return None;
        }
    }
    let mut off = header.checked_add(76)?;
    for &present in levels.iter().skip(1) {
        if present != 0 {
            let block_len = capacity.checked_mul(12)?;
            take(bytes, &mut off, block_len)?;
        }
    }
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
        if len != spec.blob_bytes {
            return None;
        }
        let raw = take(bytes, &mut off, len)?;
        let mut data = Vec::new();
        data.try_reserve_exact(len).ok()?;
        data.extend_from_slice(raw);
        records.push(SparseBlobRecord { slot, prefix, data });
    }
    if records.len() != active || u32_at(bytes, off)? != spec.end {
        return None;
    }
    Some(SparseBlobCollection {
        section_offset: section,
        capacity,
        records,
        end_offset: off,
    })
}

pub(crate) fn read(bytes: &[u8], spec: Spec) -> Option<SparseBlobCollection> {
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
