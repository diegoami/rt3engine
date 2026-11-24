//! Dependency-free RGB PNG rendering for decoded coarse heightfields.

use crate::terrain::HeightGrid;

const WATER_RGB: [u8; 3] = [41, 64, 96];

fn elevation_rgb(height: f32, low: f32, high: f32) -> [u8; 3] {
    const LOW: [u8; 3] = [58, 100, 47];
    const MID: [u8; 3] = [145, 125, 75];
    const HIGH: [u8; 3] = [235, 235, 225];
    let t = ((height - low) / (high - low).max(1.0)).clamp(0.0, 1.0);
    let (a, b, t) = if t < 0.6 {
        (LOW, MID, t / 0.6)
    } else {
        (MID, HIGH, (t - 0.6) / 0.4)
    };
    std::array::from_fn(|i| (a[i] as f32 + (b[i] as f32 - a[i] as f32) * t).round() as u8)
}

fn crc32(bytes: &[u8]) -> u32 {
    let mut crc = 0xffff_ffffu32;
    for &byte in bytes {
        crc ^= byte as u32;
        for _ in 0..8 {
            crc = (crc >> 1) ^ (0xedb8_8320 & 0u32.wrapping_sub(crc & 1));
        }
    }
    !crc
}

fn adler32(bytes: &[u8]) -> u32 {
    const MOD: u32 = 65_521;
    let mut a = 1u32;
    let mut b = 0u32;
    for &byte in bytes {
        a = (a + byte as u32) % MOD;
        b = (b + a) % MOD;
    }
    (b << 16) | a
}

fn push_chunk(out: &mut Vec<u8>, tag: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    out.extend_from_slice(tag);
    out.extend_from_slice(data);
    let mut checked = Vec::with_capacity(4 + data.len());
    checked.extend_from_slice(tag);
    checked.extend_from_slice(data);
    out.extend_from_slice(&crc32(&checked).to_be_bytes());
}

fn zlib_store(raw: &[u8]) -> Vec<u8> {
    let mut out = vec![0x78, 0x01];
    if raw.is_empty() {
        out.extend_from_slice(&[1, 0, 0, 0xff, 0xff]);
    } else {
        let count = raw.len().div_ceil(u16::MAX as usize);
        for (i, block) in raw.chunks(u16::MAX as usize).enumerate() {
            out.push((i + 1 == count) as u8);
            let len = block.len() as u16;
            out.extend_from_slice(&len.to_le_bytes());
            out.extend_from_slice(&(!len).to_le_bytes());
            out.extend_from_slice(block);
        }
    }
    out.extend_from_slice(&adler32(raw).to_be_bytes());
    out
}

/// Encodes a heightfield as a north-up, 8-bit RGB PNG. Zero and NaN are
/// water; other heights use a green-brown-white elevation gradient.
pub fn encode_png(grid: &HeightGrid) -> Vec<u8> {
    assert!(
        grid.rows > 0 && grid.cols > 0,
        "grid dimensions must be nonzero"
    );
    let width = u32::try_from(grid.cols).expect("grid width exceeds PNG");
    let height = u32::try_from(grid.rows).expect("grid height exceeds PNG");
    let mut sorted: Vec<f32> = grid
        .heights
        .iter()
        .copied()
        .filter(|v| !v.is_nan())
        .collect();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let (low, high) = if sorted.is_empty() {
        (0.0, 1.0)
    } else {
        (sorted[sorted.len() / 20], sorted[sorted.len() * 19 / 20])
    };
    let mut raw = Vec::with_capacity(grid.rows * (1 + grid.cols * 3));
    for display_row in 0..grid.rows {
        let r = grid.rows - 1 - display_row;
        raw.push(0); // PNG filter: None
        for c in 0..grid.cols {
            let v = grid.get(r, c);
            let rgb = if v.is_nan() || v == 0.0 {
                WATER_RGB
            } else {
                elevation_rgb(v, low, high)
            };
            raw.extend_from_slice(&rgb);
        }
    }
    let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
    let mut ihdr = Vec::with_capacity(13);
    ihdr.extend_from_slice(&width.to_be_bytes());
    ihdr.extend_from_slice(&height.to_be_bytes());
    ihdr.extend_from_slice(&[8, 2, 0, 0, 0]);
    push_chunk(&mut png, b"IHDR", &ihdr);
    push_chunk(&mut png, b"IDAT", &zlib_store(&raw));
    push_chunk(&mut png, b"IEND", &[]);
    png
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn adler32_reference(bytes: &[u8]) -> u32 {
        let mut a = 1u64;
        let mut b = 0u64;
        for &byte in bytes {
            a = (a + byte as u64) % 65_521;
            b = (b + a) % 65_521;
        }
        ((b << 16) | a) as u32
    }

    fn decode_stored_png(png: &[u8]) -> (u32, u32, Vec<u8>) {
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        let mut off = 8;
        let mut dims = None;
        let mut idat = Vec::new();
        let mut ended = false;
        while off < png.len() {
            let len = u32::from_be_bytes(png[off..off + 4].try_into().unwrap()) as usize;
            let tag = &png[off + 4..off + 8];
            let data = &png[off + 8..off + 8 + len];
            let crc = u32::from_be_bytes(png[off + 8 + len..off + 12 + len].try_into().unwrap());
            assert_eq!(crc32(&png[off + 4..off + 8 + len]), crc);
            if tag == b"IHDR" {
                assert_eq!(len, 13);
                assert_eq!(&data[8..13], &[8, 2, 0, 0, 0]);
                dims = Some((
                    u32::from_be_bytes(data[..4].try_into().unwrap()),
                    u32::from_be_bytes(data[4..8].try_into().unwrap()),
                ));
            } else if tag == b"IDAT" {
                idat.extend_from_slice(data);
            } else if tag == b"IEND" {
                assert!(data.is_empty());
                ended = true;
            }
            off += len + 12;
        }
        assert!(ended && off == png.len());
        let (w, h) = dims.expect("IHDR");
        assert_eq!(&idat[..2], &[0x78, 0x01]);
        let mut pos = 2;
        let mut raw = Vec::new();
        loop {
            let header = idat[pos];
            assert_eq!(header & 0b110, 0, "stored deflate blocks only");
            pos += 1;
            let len = u16::from_le_bytes(idat[pos..pos + 2].try_into().unwrap()) as usize;
            let nlen = u16::from_le_bytes(idat[pos + 2..pos + 4].try_into().unwrap());
            assert_eq!(nlen, !len as u16);
            pos += 4;
            raw.extend_from_slice(&idat[pos..pos + len]);
            pos += len;
            if header & 1 != 0 {
                break;
            }
        }
        assert_eq!(pos + 4, idat.len());
        let trailer = u32::from_be_bytes(idat[pos..pos + 4].try_into().unwrap());
        assert_eq!(adler32_reference(&raw), trailer);
        (w, h, raw)
    }

    #[test]
    fn png_is_north_up_with_water_and_nan() {
        let grid = HeightGrid {
            offset: 0,
            rows: 2,
            cols: 3,
            heights: vec![0.0, 100.0, 200.0, 300.0, 400.0, f32::NAN],
            nan_cells: 1,
            zero_cells: 1,
        };
        let png = encode_png(&grid);
        let (w, h, raw) = decode_stored_png(&png);
        assert_eq!((w, h), (3, 2));
        assert_eq!(raw.len(), 2 * (1 + 3 * 3));
        let low = elevation_rgb(100.0, 0.0, 400.0);
        let mid = elevation_rgb(300.0, 0.0, 400.0);
        let high = elevation_rgb(400.0, 0.0, 400.0);
        // South is the top display row; north is the bottom display row.
        assert_eq!(raw[0], 0);
        assert_eq!(&raw[1..4], &mid);
        assert_eq!(&raw[4..7], &high);
        assert_eq!(&raw[7..10], &WATER_RGB);
        assert_eq!(raw[10], 0);
        assert_eq!(&raw[11..14], &WATER_RGB);
        assert_eq!(&raw[14..17], &low);
        assert_eq!(&raw[17..20], &elevation_rgb(200.0, 0.0, 400.0));
    }

    #[test]
    #[should_panic(expected = "grid dimensions must be nonzero")]
    fn rejects_zero_sized_grid() {
        let grid = HeightGrid {
            offset: 0,
            rows: 0,
            cols: 1,
            heights: Vec::new(),
            nan_cells: 0,
            zero_cells: 0,
        };
        let _ = encode_png(&grid);
    }

    #[test]
    fn encodes_every_current_map() {
        let Ok(home) = std::env::var("RT3_HOME") else {
            eprintln!("RT3_HOME unset; skipping install-backed render check");
            return;
        };
        let mut paths: Vec<_> = fs::read_dir(std::path::Path::new(&home).join("maps"))
            .unwrap()
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("gmp"))
            .filter(|p| {
                fs::read(p)
                    .ok()
                    .and_then(|b| crate::map::GmpMap::peek_version(&b))
                    == Some(9701)
            })
            .collect();
        paths.sort();
        assert_eq!(paths.len(), 40);
        for path in paths {
            let bytes = fs::read(&path).unwrap();
            let grid = crate::terrain::read_framed(&bytes)
                .unwrap_or_else(|| panic!("no frame in {}", path.display()));
            let png = encode_png(&grid);
            let (w, h, raw) = decode_stored_png(&png);
            assert_eq!((w as usize, h as usize), (grid.cols, grid.rows));
            assert_eq!(raw.len(), grid.rows * (1 + grid.cols * 3));
        }
    }

    #[test]
    fn encodes_every_legacy_map() {
        let Ok(home) = std::env::var("RT3_HOME") else {
            eprintln!("RT3_HOME unset; skipping install-backed legacy render check");
            return;
        };
        for file in [
            "maps/Pacific Coastal.gmp",
            "Data/Tutorial/Tutorial_1.gmp",
            "Data/Tutorial/Tutorial_2.gmp",
        ] {
            let path = std::path::Path::new(&home).join(file);
            let bytes = fs::read(&path).unwrap();
            assert!(matches!(
                crate::map::GmpMap::peek_version(&bytes),
                Some(9609 | 9610)
            ));
            let grid = crate::terrain::read_framed(&bytes).expect("legacy frame");
            let png = encode_png(&grid);
            let (w, h, raw) = decode_stored_png(&png);
            assert_eq!((w as usize, h as usize), (grid.cols, grid.rows));
            assert_eq!(raw.len(), grid.rows * (1 + grid.cols * 3));
        }
    }
}
