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
    /// Offset of the first ending tag, before any trailer.
    pub end_offset: usize,
    /// Raw bytes between the two ending tags when the collection carries a
    /// trailer; empty otherwise. Semantics are not inferred.
    pub trailer: Vec<u8>,
}

#[derive(Clone, Copy)]
pub(crate) struct Spec {
    pub outer: u32,
    pub inner: u32,
    pub end: u32,
    pub preamble: Option<u32>,
    /// Expected blob length, or `None` to accept each record's own length.
    pub blob_bytes: Option<usize>,
    /// A second ending tag that follows a raw trailer block.
    pub trailer_end: Option<u32>,
    /// Optional fixed header words after the `0, 6` prefix.
    pub header2: Option<u32>,
    pub header3: Option<u32>,
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
        || spec
            .header2
            .is_some_and(|value| u32_at(bytes, header + 8) != Some(value))
        || spec
            .header3
            .is_some_and(|value| u32_at(bytes, header + 12) != Some(value))
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
    // Original reader 0x00517d90 uses (capacity + 8) >> 3, so exact
    // multiples of eight still allocate a following mask byte.
    let mask_bytes = capacity.checked_add(8)? >> 3;
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
        if let Some(expected) = spec.blob_bytes {
            if len != expected {
                return None;
            }
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
    let end_offset = off;
    let mut trailer = Vec::new();
    if let Some(end_tag) = spec.trailer_end {
        let start = off.checked_add(4)?;
        let window = bytes.get(start..)?;
        let limit = window.len().min(64 * 1024);
        let tag = end_tag.to_le_bytes();
        let pos = window[..limit].windows(4).position(|w| w == tag)?;
        trailer.extend_from_slice(&window[..pos]);
    }
    Some(SparseBlobCollection {
        section_offset: section,
        capacity,
        records,
        end_offset,
        trailer,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn push_u32(out: &mut Vec<u8>, value: u32) {
        out.extend_from_slice(&value.to_le_bytes());
    }

    /// Header and level table for a collection with `capacity` slots and no
    /// level payload blocks.
    fn collection_without_payloads(spec: Spec, capacity: u32, active: u32) -> Vec<u8> {
        let mut bytes = vec![0u8; 0x3000];
        push_u32(&mut bytes, spec.outer);
        push_u32(&mut bytes, spec.inner);
        if let Some(preamble) = spec.preamble {
            push_u32(&mut bytes, preamble);
        }
        // Header: fixed words, capacity, active count, trailing zero.
        for word in [0, 6, 10, 20, capacity, active, 0] {
            push_u32(&mut bytes, word);
        }
        for level in 0..4 {
            push_u32(&mut bytes, u32::from(level == 0));
            push_u32(&mut bytes, 0);
            push_u32(&mut bytes, 0);
        }
        bytes
    }

    fn sample_spec() -> Spec {
        Spec {
            outer: 0x32dc,
            inner: 0x4e99,
            end: 0x4e9a,
            preamble: Some(1001),
            blob_bytes: Some(4),
            trailer_end: None,
            header2: Some(10),
            header3: Some(20),
        }
    }

    /// One collection with a preamble, an active level-2 payload block, a
    /// one-byte mask and two active records in slots 1 and 3.
    fn sample_with_preamble() -> (Spec, Vec<u8>, usize) {
        let spec = sample_spec();
        let mut bytes = collection_without_payloads(spec, 4, 2);
        let base = 0x3000;
        let level2 = bytes.len() - 48 + 2 * 12;
        bytes[level2..level2 + 4].copy_from_slice(&1u32.to_le_bytes());
        bytes.resize(bytes.len() + 4 * 12, 0); // level-2 payload: capacity × 12
        bytes.push(0b101); // bits set mark absent; slots 1 and 3 are active
        for value in [0x11u8, 0x33] {
            bytes.extend_from_slice(&[value; 6]);
            bytes.extend_from_slice(&4u16.to_le_bytes());
            bytes.extend_from_slice(&[value; 4]);
        }
        push_u32(&mut bytes, spec.end);
        (spec, bytes, base)
    }

    #[test]
    fn rejects_truncated_and_malformed_sparse_collections() {
        let (spec, bytes, base) = sample_with_preamble();
        let collection = read(&bytes, spec).expect("sample collection");
        assert_eq!(collection.capacity, 4);
        assert_eq!(collection.records.len(), 2);
        assert_eq!(collection.records[0].slot, 1);
        assert_eq!(collection.records[1].slot, 3);

        for end in 0..bytes.len() {
            assert!(read(&bytes[..end], spec).is_none(), "accepted prefix {end}");
        }

        // Offsets into `sample_with_preamble`: header at base+12, levels at
        // base+40, level-2 payload at base+88, mask at base+136.
        let wrong_words: [(&str, usize, u32); 6] = [
            ("outer tag", base, 0x32dd),
            ("inner tag", base + 4, 0x4e98),
            ("preamble", base + 8, 1002),
            ("level 0 flag", base + 40, 0),
            ("level 2 flag", base + 64, 2),
            ("terminator", bytes.len() - 4, 0),
        ];
        for (what, at, value) in wrong_words {
            let mut broken = bytes.clone();
            broken[at..at + 4].copy_from_slice(&value.to_le_bytes());
            assert!(read(&broken, spec).is_none(), "accepted wrong {what}");
        }
    }

    #[test]
    fn rejects_bad_counts_wrong_blob_length_and_oversized_capacity() {
        let spec = sample_spec();

        // All four mask bits set: every slot absent, the terminator immediately
        // after the mask, while the header still declares two active records.
        // Only the final record-count check can reject this buffer.
        let mut under_count = collection_without_payloads(spec, 4, 2);
        under_count.push(0x0f);
        push_u32(&mut under_count, spec.end);
        assert!(
            read(&under_count, spec).is_none(),
            "accepted a mask with fewer records than declared"
        );

        // One active slot declaring five payload bytes while the spec wants
        // four; the five bytes exist, so only the length check rejects it.
        let mut wrong_length = collection_without_payloads(spec, 1, 1);
        wrong_length.push(0x00); // bit clear: slot 0 active
        wrong_length.extend_from_slice(&[0x44; 6]);
        wrong_length.extend_from_slice(&5u16.to_le_bytes());
        wrong_length.extend_from_slice(&[0x44; 5]);
        push_u32(&mut wrong_length, spec.end);
        assert!(
            read(&wrong_length, spec).is_none(),
            "accepted a five-byte blob"
        );

        // One slot over MAX_SLOTS with a complete mask of absent slots: the
        // slot limit must fire before the 129 mask bytes are considered.
        let mut oversized = collection_without_payloads(spec, 1025, 0);
        oversized.resize(oversized.len() + 129, 0xff);
        push_u32(&mut oversized, spec.end);
        assert!(
            read(&oversized, spec).is_none(),
            "accepted capacity over the slot limit"
        );

        // Active above capacity is rejected, but the final record count also
        // rejects it; this is regression coverage, not an isolated guard.
        let mut active_over = collection_without_payloads(spec, 4, 5);
        active_over.push(0x0f);
        push_u32(&mut active_over, spec.end);
        assert!(
            read(&active_over, spec).is_none(),
            "accepted more active records than slots"
        );
    }

    #[test]
    fn exact_eight_capacity_allocates_two_mask_bytes() {
        let spec = Spec {
            outer: 0x32dc,
            inner: 0x4e99,
            end: 0x4e9a,
            preamble: None,
            blob_bytes: Some(4),
            trailer_end: None,
            header2: Some(10),
            header3: Some(20),
        };
        let mut bytes = vec![0u8; 0x3000];
        push_u32(&mut bytes, spec.outer);
        push_u32(&mut bytes, spec.inner);
        for word in [0, 6, 10, 20, 8, 1, 0] {
            push_u32(&mut bytes, word);
        }
        for level in 0..4 {
            push_u32(&mut bytes, if level == 0 { 1 } else { 0 });
            push_u32(&mut bytes, 0);
            push_u32(&mut bytes, 0);
        }
        bytes.extend([0xfe, 0xff]); // slot 0 active; original allocates two bytes
        bytes.extend_from_slice(&[7; 6]);
        bytes.extend_from_slice(&4u16.to_le_bytes());
        bytes.extend_from_slice(&[1, 2, 3, 4]);
        push_u32(&mut bytes, spec.end);

        let collection = read(&bytes, spec).expect("exact-eight sparse collection");
        assert_eq!(collection.capacity, 8);
        assert_eq!(collection.records.len(), 1);
        assert_eq!(collection.records[0].slot, 0);
        assert_eq!(collection.records[0].data, [1, 2, 3, 4]);
        assert_eq!(collection.end_offset + 4, bytes.len());
        for end in 0..bytes.len() {
            assert!(read(&bytes[..end], spec).is_none(), "accepted prefix {end}");
        }
    }
}
