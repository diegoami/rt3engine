//! Reader for Railroad Tycoon 3 `PK4` packed asset archives.
//!
//! Layout (format version 1003, the only variant in the verified GOG
//! installation; see `docs/evidence/pk4-format-v1.md`):
//!
//! ```text
//! header:  u32 version (=1003)   u32 count
//! entries: count × 74 bytes:
//!          u32 hash      lookup key used by the game (32 buckets, hash & 0x1F)
//!          u32 size      stored payload size in bytes
//!          u32 offset    payload start, relative to the end of the entry table
//!          u8  flag      0 in every shipped archive; meaning unverified
//!          char name[61] NUL-padded entry name
//! data:    payload blocks at the given offsets (stored raw; no compression
//!          or encryption is applied — DDS/WAV/TGA payloads keep their magic)
//! ```
//!
//! The structure was decoded from `RT3.exe` (index loader at `0x0053b090`,
//! seek formula `8 + 74*count + offset` at `0x0053bd40`) and validated
//! against all 18 shipped archives: zero out-of-bounds or overlapping
//! payloads and no gaps. Format versions below 1003 use 13-byte entries
//! without names and are rejected here; none ship with the game.
//!
//! The name→hash algorithm is not yet identified (not a CRC32 or the common
//! string hashes); it is not needed to read archives, only to author them.

use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};

/// Format version this reader implements.
pub const FORMAT_VERSION: u32 = 1003;
/// Size of one table-of-contents entry in bytes.
pub const ENTRY_SIZE: usize = 74;
/// Length of the fixed, NUL-padded name field.
pub const NAME_LEN: usize = 61;
/// Offset of the name field inside an entry.
const NAME_OFFSET: usize = 13;

/// An error produced while reading a PK4 archive.
#[derive(Debug)]
pub enum Pk4Error {
    /// The file could not be opened or read.
    Io(PathBuf, io::Error),
    /// The file is smaller than its own header or table.
    Truncated {
        path: PathBuf,
        size: u64,
        expected: u64,
    },
    /// Unsupported format version (we implement 1003 only).
    UnsupportedVersion { path: PathBuf, version: u32 },
    /// An entry payload lies outside the archive or overlaps another entry.
    BadEntry {
        path: PathBuf,
        index: usize,
        name: String,
        detail: String,
    },
    /// An entry name is not valid printable text.
    BadName { path: PathBuf, index: usize },
}

impl std::fmt::Display for Pk4Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Pk4Error::Io(path, error) => write!(f, "reading {}: {error}", path.display()),
            Pk4Error::Truncated {
                path,
                size,
                expected,
            } => {
                write!(
                    f,
                    "{}: truncated ({size} bytes, need {expected})",
                    path.display()
                )
            }
            Pk4Error::UnsupportedVersion { path, version } => {
                write!(
                    f,
                    "{}: unsupported PK4 version {version} (only {FORMAT_VERSION})",
                    path.display()
                )
            }
            Pk4Error::BadEntry {
                path,
                index,
                name,
                detail,
            } => {
                write!(f, "{}: entry {index} {name:?}: {detail}", path.display())
            }
            Pk4Error::BadName { path, index } => {
                write!(f, "{}: entry {index} has an unusable name", path.display())
            }
        }
    }
}

impl std::error::Error for Pk4Error {}

/// One table-of-contents entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    /// The game's lookup key for this entry (algorithm not yet identified).
    pub hash: u32,
    /// Stored payload size in bytes.
    pub size: u32,
    /// Payload start relative to the end of the entry table.
    pub offset: u32,
    /// The single trailing byte of the record; 0 in every shipped archive.
    pub flag: u8,
    /// The entry name, decoded from the fixed NUL-padded field.
    pub name: String,
}

/// An opened PK4 archive.
#[derive(Debug)]
pub struct Pk4Archive {
    path: PathBuf,
    version: u32,
    entries: Vec<Entry>,
    /// Absolute file offset where the payload region starts.
    data_start: u64,
}

impl Pk4Archive {
    /// Opens and fully validates the archive at `path`.
    pub fn open(path: &Path) -> Result<Pk4Archive, Pk4Error> {
        let bytes = fs::read(path).map_err(|e| Pk4Error::Io(path.to_path_buf(), e))?;
        Self::from_bytes(path, &bytes)
    }

    /// Parses an archive image held in memory.
    pub fn from_bytes(path: &Path, bytes: &[u8]) -> Result<Pk4Archive, Pk4Error> {
        if bytes.len() < 8 {
            return Err(Pk4Error::Truncated {
                path: path.to_path_buf(),
                size: bytes.len() as u64,
                expected: 8,
            });
        }
        let version = u32::from_le_bytes(bytes[0..4].try_into().unwrap());
        let count = u32::from_le_bytes(bytes[4..8].try_into().unwrap()) as usize;
        if version != FORMAT_VERSION {
            return Err(Pk4Error::UnsupportedVersion {
                path: path.to_path_buf(),
                version,
            });
        }
        let table_end = 8u64 + ENTRY_SIZE as u64 * count as u64;
        if (bytes.len() as u64) < table_end {
            return Err(Pk4Error::Truncated {
                path: path.to_path_buf(),
                size: bytes.len() as u64,
                expected: table_end,
            });
        }
        let mut entries = Vec::with_capacity(count);
        for index in 0..count {
            let base = 8 + ENTRY_SIZE * index;
            let record = &bytes[base..base + ENTRY_SIZE];
            let hash = u32::from_le_bytes(record[0..4].try_into().unwrap());
            let size = u32::from_le_bytes(record[4..8].try_into().unwrap());
            let offset = u32::from_le_bytes(record[8..12].try_into().unwrap());
            let flag = record[12];
            let name_field = &record[NAME_OFFSET..NAME_OFFSET + NAME_LEN];
            let nul = name_field.iter().position(|b| *b == 0).unwrap_or(NAME_LEN);
            let name_bytes = &name_field[..nul];
            if name_bytes.is_empty() || !name_bytes.iter().all(u8::is_ascii_graphic) {
                return Err(Pk4Error::BadName {
                    path: path.to_path_buf(),
                    index,
                });
            }
            entries.push(Entry {
                hash,
                size,
                offset,
                flag,
                name: String::from_utf8_lossy(name_bytes).into_owned(),
            });
        }
        let archive = Pk4Archive {
            path: path.to_path_buf(),
            version,
            data_start: table_end,
            entries,
        };
        archive.validate_bounds(bytes.len() as u64)?;
        Ok(archive)
    }

    fn validate_bounds(&self, file_len: u64) -> Result<(), Pk4Error> {
        let mut spans: Vec<(u64, u64, usize)> = self
            .entries
            .iter()
            .enumerate()
            .map(|(index, e)| (self.data_start + e.offset as u64, e.size as u64, index))
            .collect();
        spans.sort_unstable();
        let mut prev_end = self.data_start;
        for (start, size, index) in spans {
            if start < prev_end {
                return Err(Pk4Error::BadEntry {
                    path: self.path.clone(),
                    index,
                    name: self.entries[index].name.clone(),
                    detail: format!(
                        "payload at {start:#x} overlaps the previous end {prev_end:#x}"
                    ),
                });
            }
            let end = start + size;
            if end > file_len {
                return Err(Pk4Error::BadEntry {
                    path: self.path.clone(),
                    index,
                    name: self.entries[index].name.clone(),
                    detail: format!(
                        "payload [{start:#x},{end:#x}) exceeds the file size {file_len:#x}"
                    ),
                });
            }
            prev_end = end;
        }
        Ok(())
    }

    /// The archive path as opened.
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The format version word (always 1003 for this reader).
    pub fn version(&self) -> u32 {
        self.version
    }

    /// All entries, in table order.
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }

    /// Absolute file offset where an entry's payload starts.
    pub fn payload_offset(&self, entry: &Entry) -> u64 {
        self.data_start + entry.offset as u64
    }

    /// Finds an entry by name, ASCII case-insensitively (the game compares
    /// with `__stricmp`).
    pub fn find(&self, name: &str) -> Option<&Entry> {
        self.entries
            .iter()
            .find(|e| e.name.eq_ignore_ascii_case(name))
    }

    /// Reads an entry's payload into a vector.
    pub fn read(&self, entry: &Entry) -> Result<Vec<u8>, Pk4Error> {
        let mut file =
            fs::File::open(&self.path).map_err(|e| Pk4Error::Io(self.path.clone(), e))?;
        read_payload(&mut file, self.payload_offset(entry), entry.size)
            .map_err(|e| Pk4Error::Io(self.path.clone(), e))
    }
}

/// Reads `size` bytes starting at `offset` from an already-opened file.
fn read_payload(file: &mut fs::File, offset: u64, size: u32) -> io::Result<Vec<u8>> {
    use std::io::Seek;
    file.seek(std::io::SeekFrom::Start(offset))?;
    let mut buffer = vec![0u8; size as usize];
    file.read_exact(&mut buffer)?;
    Ok(buffer)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds a minimal v1003 archive image from (name, payload) pairs.
    fn build_archive(pairs: &[(&str, &[u8])]) -> Vec<u8> {
        let count = pairs.len();
        let mut out = Vec::new();
        out.extend_from_slice(&1003u32.to_le_bytes());
        out.extend_from_slice(&(count as u32).to_le_bytes());
        let mut data = Vec::new();
        for (name, payload) in pairs {
            let mut record = [0u8; ENTRY_SIZE];
            record[0..4].copy_from_slice(&0xDEADBEEFu32.to_le_bytes()); // hash: any value to read
            record[4..8].copy_from_slice(&(payload.len() as u32).to_le_bytes());
            record[8..12].copy_from_slice(&(data.len() as u32).to_le_bytes());
            // flag stays 0
            record[NAME_OFFSET..NAME_OFFSET + name.len()].copy_from_slice(name.as_bytes());
            out.extend_from_slice(&record);
            data.extend_from_slice(payload);
        }
        out.extend_from_slice(&data);
        out
    }

    fn tmp_path(tag: &str) -> PathBuf {
        std::env::temp_dir()
            .join("rt3-kit-pk4-tests")
            .join(format!("{tag}-{}", std::process::id()))
    }

    #[test]
    fn parses_and_reads_entries() {
        let image = build_archive(&[
            ("a.dds", b"DDS payload one"),
            ("b.win", b"payload two, longer"),
        ]);
        let path = PathBuf::from("fixture-a.pk4");
        let archive = Pk4Archive::from_bytes(&path, &image).unwrap();
        assert_eq!(archive.version(), 1003);
        assert_eq!(archive.entries().len(), 2);
        let entry = archive.find("A.DDS").expect("case-insensitive find");
        assert_eq!(entry.name, "a.dds");
        assert_eq!(entry.size as usize, b"DDS payload one".len());
        // from_bytes cannot read payloads from disk; check offsets instead
        assert_eq!(archive.payload_offset(entry), archive.data_start);
        let second = archive.find("b.win").unwrap();
        assert_eq!(
            archive.payload_offset(second),
            archive.data_start + b"DDS payload one".len() as u64
        );
    }

    #[test]
    fn empty_archive_is_header_only() {
        let mut image = Vec::new();
        image.extend_from_slice(&1003u32.to_le_bytes());
        image.extend_from_slice(&0u32.to_le_bytes());
        let archive = Pk4Archive::from_bytes(Path::new("empty.pk4"), &image).unwrap();
        assert!(archive.entries().is_empty());
        assert_eq!(archive.data_start, 8);
    }

    #[test]
    fn rejects_other_versions_and_truncation() {
        let mut image = vec![0u8; 16];
        image[0..4].copy_from_slice(&1002u32.to_le_bytes());
        assert!(matches!(
            Pk4Archive::from_bytes(Path::new("old.pk4"), &image),
            Err(Pk4Error::UnsupportedVersion { version: 1002, .. })
        ));
        assert!(matches!(
            Pk4Archive::from_bytes(Path::new("tiny.pk4"), &[1, 2, 3]),
            Err(Pk4Error::Truncated { .. })
        ));
    }

    #[test]
    fn rejects_out_of_bounds_payload() {
        let mut image = build_archive(&[("x.dds", b"12345")]);
        // Corrupt the size field of entry 0 to point past the end.
        let count = u32::from_le_bytes(image[4..8].try_into().unwrap()) as usize;
        assert_eq!(count, 1);
        let record = &mut image[8..8 + ENTRY_SIZE];
        record[4..8].copy_from_slice(&0xFFFF_FFF0u32.to_le_bytes());
        assert!(matches!(
            Pk4Archive::from_bytes(Path::new("bad.pk4"), &image),
            Err(Pk4Error::BadEntry { .. })
        ));
    }

    #[test]
    fn round_trips_through_disk() {
        let path = tmp_path("roundtrip");
        std::fs::create_dir_all(&path).unwrap();
        let file = path.join("sample.pk4");
        let image = build_archive(&[("hello.tga", b"TGA-BYTES"), ("two.wav", b"RIFFxx")]);
        std::fs::write(&file, &image).unwrap();
        let archive = Pk4Archive::open(&file).unwrap();
        let payload = archive.read(archive.find("hello.tga").unwrap()).unwrap();
        assert_eq!(payload, b"TGA-BYTES");
        let payload = archive.read(archive.find("TWO.WAV").unwrap()).unwrap();
        assert_eq!(payload, b"RIFFxx");
        let _ = std::fs::remove_dir_all(&path);
    }

    /// Install-backed check: parses every shipped archive when the original
    /// installation is available. Self-skips without `RT3_HOME`.
    #[test]
    fn parses_every_shipped_archive() {
        let Ok(home) = std::env::var("RT3_HOME") else {
            eprintln!("RT3_HOME unset; skipping install-backed PK4 check");
            return;
        };
        let mut found = 0;
        let mut entries = 0usize;
        for dir in ["Data/2D", "Data/3D", "Data/Sound"] {
            let dir = Path::new(&home).join(dir);
            let Ok(read_dir) = fs::read_dir(&dir) else {
                continue;
            };
            let mut paths: Vec<_> = read_dir.filter_map(|e| e.ok().map(|e| e.path())).collect();
            paths.sort();
            for path in paths {
                if path
                    .extension()
                    .and_then(|e| e.to_str())
                    .map(|e| e.eq_ignore_ascii_case("pk4"))
                    != Some(true)
                {
                    continue;
                }
                let archive = Pk4Archive::open(&path).unwrap_or_else(|e| panic!("{e}"));
                entries += archive.entries().len();
                found += 1;
            }
        }
        assert_eq!(found, 18, "expected the 18 shipped archives");
        assert_eq!(entries, 8375, "total shipped entries");
    }
}
