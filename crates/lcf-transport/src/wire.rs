//! Versioned compact binary envelope codec with strict bounds.
use lcf_core::Envelope;
use std::io;

const MAGIC: &[u8;4]=b"LCF1";
const MAX_IDENTITY:usize=128;
const MAX_PAYLOAD:usize=65_507;

fn put_string(out:&mut Vec<u8>,s:&str)->io::Result<()> {
    if s.len()>MAX_IDENTITY { return Err(io::Error::new(io::ErrorKind::InvalidInput,"identity too long")); }
    out.extend_from_slice(&(s.len() as u16).to_be_bytes()); out.extend_from_slice(s.as_bytes()); Ok(())
}
fn take<'a>(data:&'a [u8],pos:&mut usize,n:usize)->io::Result<&'a [u8]> {
    let end=pos.checked_add(n).ok_or_else(||io::Error::new(io::ErrorKind::InvalidData,"length overflow"))?;
    if end>data.len(){return Err(io::Error::new(io::ErrorKind::UnexpectedEof,"truncated frame"));}
    let s=&data[*pos..end]; *pos=end; Ok(s)
}
fn read_string(data:&[u8],pos:&mut usize)->io::Result<String> {
    let n=u16::from_be_bytes(take(data,pos,2)?.try_into().unwrap()) as usize;
    if n>MAX_IDENTITY{return Err(io::Error::new(io::ErrorKind::InvalidData,"identity too long"));}
    String::from_utf8(take(data,pos,n)?.to_vec()).map_err(|_|io::Error::new(io::ErrorKind::InvalidData,"invalid utf-8"))
}

pub fn encode(e:&Envelope)->io::Result<Vec<u8>> {
    if e.payload.len()>MAX_PAYLOAD{return Err(io::Error::new(io::ErrorKind::InvalidInput,"payload too large"));}
    let mut out=Vec::with_capacity(64+e.payload.len()); out.extend_from_slice(MAGIC);
    out.extend_from_slice(&e.id); put_string(&mut out,&e.source)?; put_string(&mut out,&e.destination)?;
    out.extend_from_slice(&e.created_at_ms.to_be_bytes()); out.extend_from_slice(&e.expires_at_ms.to_be_bytes());
    out.push(e.hop_limit); out.extend_from_slice(&(e.payload.len() as u32).to_be_bytes()); out.extend_from_slice(&e.payload);
    Ok(out)
}

pub fn decode(data:&[u8])->io::Result<Envelope> {
    let mut p=0;
    if take(data,&mut p,4)?!=MAGIC{return Err(io::Error::new(io::ErrorKind::InvalidData,"bad LCF magic/version"));}
    let id: [u8;16]=take(data,&mut p,16)?.try_into().unwrap();
    let source=read_string(data,&mut p)?; let destination=read_string(data,&mut p)?;
    let created_at_ms=u64::from_be_bytes(take(data,&mut p,8)?.try_into().unwrap());
    let expires_at_ms=u64::from_be_bytes(take(data,&mut p,8)?.try_into().unwrap());
    let hop_limit=take(data,&mut p,1)?[0];
    let n=u32::from_be_bytes(take(data,&mut p,4)?.try_into().unwrap()) as usize;
    if n>MAX_PAYLOAD{return Err(io::Error::new(io::ErrorKind::InvalidData,"payload exceeds limit"));}
    let payload=take(data,&mut p,n)?.to_vec();
    if p!=data.len(){return Err(io::Error::new(io::ErrorKind::InvalidData,"trailing bytes"));}
    Ok(Envelope{id,source,destination,created_at_ms,expires_at_ms,hop_limit,payload})
}

#[cfg(test)]
mod tests {
 use super::*;
 #[test] fn round_trip() {
  let e=Envelope{id:[9;16],source:"a".into(),destination:"b".into(),created_at_ms:1,expires_at_ms:9,hop_limit:4,payload:b"hi".to_vec()};
  assert_eq!(decode(&encode(&e).unwrap()).unwrap(),e);
 }
 #[test] fn rejects_truncated_packet() { assert!(decode(b"LCF1").is_err()); }
}
