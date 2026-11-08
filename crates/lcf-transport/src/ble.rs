//! BLE GATT framing contract. OS/MCU radio APIs plug into BleFrameIo.
use std::io;

pub const LCF_BLE_SERVICE_UUID:&str="7b8a0001-6e5a-4c46-9c46-4c4346000001";
pub const LCF_BLE_TX_UUID:&str="7b8a0002-6e5a-4c46-9c46-4c4346000001";
pub const LCF_BLE_RX_UUID:&str="7b8a0003-6e5a-4c46-9c46-4c4346000001";

/// Platform adapter contract: implement with CoreBluetooth, Android BLE or
/// the selected embedded BLE stack. A discovered peripheral is not trusted.
pub trait BleFrameIo {
    fn write_characteristic(&mut self,peripheral:&str,characteristic:&str,bytes:&[u8])->io::Result<()>;
    fn read_frame(&mut self,timeout_ms:u32)->io::Result<Option<Vec<u8>>>;
}

/// Split a frame into ATT-value-sized chunks. Caller supplies negotiated MTU.
pub fn fragment(frame:&[u8],att_mtu:usize)->io::Result<Vec<Vec<u8>>> {
    let chunk=att_mtu.checked_sub(3).filter(|n|*n>4)
        .ok_or_else(||io::Error::new(io::ErrorKind::InvalidInput,"ATT MTU too small"))?;
    let data_chunk=chunk-4;
    let count=(frame.len()+data_chunk-1)/data_chunk;
    if count>u16::MAX as usize{return Err(io::Error::new(io::ErrorKind::InvalidInput,"too many fragments"));}
    let mut out=Vec::with_capacity(count);
    for (i,part) in frame.chunks(data_chunk).enumerate() {
        let mut packet=Vec::with_capacity(part.len()+4);
        packet.extend_from_slice(&(i as u16).to_be_bytes());
        packet.extend_from_slice(&(count as u16).to_be_bytes());
        packet.extend_from_slice(part); out.push(packet);
    }
    Ok(out)
}

/// Reassemble one ordered frame; rejects missing, duplicate or inconsistent fragments.
pub fn reassemble(parts:&[Vec<u8>])->io::Result<Vec<u8>> {
    if parts.is_empty(){return Err(io::Error::new(io::ErrorKind::InvalidInput,"no fragments"));}
    let count=u16::from_be_bytes([parts[0][2],parts[0][3]]) as usize;
    if count!=parts.len(){return Err(io::Error::new(io::ErrorKind::InvalidData,"fragment count mismatch"));}
    let mut out=Vec::new();
    for (expected,p) in parts.iter().enumerate() {
        if p.len()<4 || u16::from_be_bytes([p[0],p[1]]) as usize!=expected ||
           u16::from_be_bytes([p[2],p[3]]) as usize!=count {
            return Err(io::Error::new(io::ErrorKind::InvalidData,"fragment sequence invalid"));
        }
        out.extend_from_slice(&p[4..]);
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
 use super::*;
 #[test] fn fragments_round_trip_at_mtu() {
  let frame=vec![0x5a;500]; let pieces=fragment(&frame,23).unwrap();
  assert!(pieces.iter().all(|p|p.len()<=20));
  assert_eq!(reassemble(&pieces).unwrap(),frame);
 }
}
