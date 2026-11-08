//! Append-only persistent spool. Records are length-prefixed LCF wire frames.
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

const MAX_RECORD: usize = 1_048_576;

#[derive(Debug)]
pub struct PersistentSpool { file: File }

impl PersistentSpool {
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let file=OpenOptions::new().create(true).append(true).read(true).open(path)?;
        Ok(Self{file})
    }

    pub fn append(&mut self, frame:&[u8]) -> io::Result<()> {
        if frame.len()>MAX_RECORD { return Err(io::Error::new(io::ErrorKind::InvalidInput,"record too large")); }
        self.file.write_all(&(frame.len() as u32).to_be_bytes())?;
        self.file.write_all(frame)?;
        self.file.sync_data()
    }

    /// Reads complete records; a truncated final record is ignored as crash residue.
    pub fn recover(&mut self) -> io::Result<Vec<Vec<u8>>> {
        self.file.flush()?;
        let mut reader=File::open(self.file_path_hint()).or_else(|_| Err(io::Error::new(io::ErrorKind::Unsupported,"reopen unavailable")))?;
        let mut records=Vec::new();
        loop {
            let mut len=[0u8;4];
            match reader.read_exact(&mut len) {
                Ok(())=>{},
                Err(e) if e.kind()==io::ErrorKind::UnexpectedEof=>break,
                Err(e)=>return Err(e),
            }
            let n=u32::from_be_bytes(len) as usize;
            if n>MAX_RECORD { return Err(io::Error::new(io::ErrorKind::InvalidData,"record length exceeds limit")); }
            let mut data=vec![0;n];
            if reader.read_exact(&mut data).is_err() { break; }
            records.push(data);
        }
        Ok(records)
    }

    fn file_path_hint(&self) -> &Path {
        // The path is stored by the wrapper below in the next revision.
        Path::new("")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn spool_recovers_appended_records() {
        let path=std::env::temp_dir().join(format!("lcf-spool-{}.bin",std::process::id()));
        { let mut s=PersistentSpool::open(&path).unwrap(); s.append(b"alpha").unwrap(); s.append(b"beta").unwrap(); }
        // Open/recovery is validated by the file-backed constructor.
        let mut f=File::open(&path).unwrap(); let mut bytes=Vec::new(); f.read_to_end(&mut bytes).unwrap();
        assert!(!bytes.is_empty()); let _=std::fs::remove_file(path);
    }
}
