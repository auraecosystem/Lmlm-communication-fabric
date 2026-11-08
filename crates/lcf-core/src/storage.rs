//! Append-only persistent spool. Records are length-prefixed LCF wire frames.
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

const MAX_RECORD: usize = 1_048_576;

#[derive(Debug)]
pub struct PersistentSpool {
    file: File,
    path: PathBuf,
}

impl PersistentSpool {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let file = OpenOptions::new().create(true).append(true).read(true).open(&path)?;
        Ok(Self { file, path })
    }

    pub fn append(&mut self, frame: &[u8]) -> io::Result<()> {
        if frame.len() > MAX_RECORD {
            return Err(io::Error::new(io::ErrorKind::InvalidInput, "record too large"));
        }
        self.file.write_all(&(frame.len() as u32).to_be_bytes())?;
        self.file.write_all(frame)?;
        self.file.sync_data()
    }

    /// Reads complete records; a truncated final record is ignored as crash residue.
    pub fn recover(&mut self) -> io::Result<Vec<Vec<u8>>> {
        self.file.flush()?;
        let mut reader = File::open(&self.path)?;
        let mut records = Vec::new();
        loop {
            let mut len = [0u8; 4];
            match reader.read_exact(&mut len) {
                Ok(()) => {}
                Err(e) if e.kind() == io::ErrorKind::UnexpectedEof => break,
                Err(e) => return Err(e),
            }
            let n = u32::from_be_bytes(len) as usize;
            if n > MAX_RECORD {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "record length exceeds limit"));
            }
            let mut data = vec![0; n];
            if reader.read_exact(&mut data).is_err() { break; }
            records.push(data);
        }
        Ok(records)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spool_recovers_records_after_reopen() {
        let path = std::env::temp_dir().join(format!("lcf-spool-{}.bin", std::process::id()));
        {
            let mut spool = PersistentSpool::open(&path).unwrap();
            spool.append(b"alpha").unwrap();
            spool.append(b"beta").unwrap();
            assert_eq!(spool.recover().unwrap(), vec![b"alpha".to_vec(), b"beta".to_vec()]);
        }
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn ignores_truncated_final_record() {
        let path = std::env::temp_dir().join(format!("lcf-spool-truncated-{}.bin", std::process::id()));
        {
            let mut spool = PersistentSpool::open(&path).unwrap();
            spool.append(b"complete").unwrap();
        }
        OpenOptions::new().append(true).open(&path).unwrap().write_all(&[0, 0, 0, 5, 1, 2]).unwrap();
        let mut spool = PersistentSpool::open(&path).unwrap();
        assert_eq!(spool.recover().unwrap(), vec![b"complete".to_vec()]);
        let _ = std::fs::remove_file(path);
    }
}
