//! Authenticated packet protection and signed peer statements.
//!
//! Session keys and nonce prefixes must be provisioned by a trusted key-management
//! layer. Use a distinct random 32-bit nonce prefix for each direction and never
//! reuse a (key, prefix, counter) tuple. This module intentionally does not claim
//! to implement an authenticated key exchange.
use chacha20poly1305::{aead::{Aead, Payload}, ChaCha20Poly1305, KeyInit, Nonce};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};

const AAD: &[u8] = b"LMLM-LCF/1 secure packet";
const MAX_PEER_ID: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SecurityError {
    InvalidPeerId,
    InvalidKey,
    InvalidSignature,
    ExpiredStatement,
    MalformedPacket,
    WrongSession,
    Replay,
    CounterExhausted,
    AuthenticationFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignedPeerStatement {
    pub peer_id: String,
    pub sequence: u64,
    pub expires_at_ms: u64,
    pub signature: [u8; 64],
}

impl SignedPeerStatement {
    fn signing_bytes(peer_id: &str, sequence: u64, expires_at_ms: u64) -> Result<Vec<u8>, SecurityError> {
        if peer_id.is_empty() || peer_id.len() > MAX_PEER_ID {
            return Err(SecurityError::InvalidPeerId);
        }
        let mut out = Vec::with_capacity(2 + peer_id.len() + 16);
        out.extend_from_slice(&(peer_id.len() as u16).to_be_bytes());
        out.extend_from_slice(peer_id.as_bytes());
        out.extend_from_slice(&sequence.to_be_bytes());
        out.extend_from_slice(&expires_at_ms.to_be_bytes());
        Ok(out)
    }

    pub fn sign(peer_id: String, sequence: u64, expires_at_ms: u64, key: &SigningKey) -> Result<Self, SecurityError> {
        let bytes = Self::signing_bytes(&peer_id, sequence, expires_at_ms)?;
        let signature = key.sign(&bytes).to_bytes();
        Ok(Self { peer_id, sequence, expires_at_ms, signature })
    }

    pub fn verify(&self, key: &VerifyingKey, now_ms: u64) -> Result<(), SecurityError> {
        if now_ms >= self.expires_at_ms {
            return Err(SecurityError::ExpiredStatement);
        }
        let bytes = Self::signing_bytes(&self.peer_id, self.sequence, self.expires_at_ms)?;
        let signature = Signature::from_bytes(&self.signature);
        key.verify(&bytes, &signature).map_err(|_| SecurityError::InvalidSignature)
    }
}

#[derive(Debug, Default)]
struct ReplayWindow {
    highest: Option<u64>,
    bitmap: u64,
}

impl ReplayWindow {
    fn would_accept(&self, counter: u64) -> bool {
        match self.highest {
            None => true,
            Some(highest) if counter > highest => true,
            Some(highest) => {
                let distance = highest - counter;
                distance < 64 && (self.bitmap & (1u64 << distance)) == 0
            }
        }
    }

    fn commit(&mut self, counter: u64) {
        match self.highest {
            None => { self.highest = Some(counter); self.bitmap = 1; }
            Some(highest) if counter > highest => {
                let shift = counter - highest;
                self.bitmap = if shift >= 64 { 1 } else { (self.bitmap << shift) | 1 };
                self.highest = Some(counter);
            }
            Some(highest) => self.bitmap |= 1u64 << (highest - counter),
        }
    }
}

/// One directional AEAD session. The caller must provision a unique nonce prefix
/// per direction for this key, persist/rekey before counter exhaustion, and never
/// restore an old counter with the same key and prefix.
pub struct SessionCipher {
    cipher: ChaCha20Poly1305,
    nonce_prefix: [u8; 4],
    tx_counter: u64,
    rx_window: ReplayWindow,
}

impl SessionCipher {
    pub fn new(key: [u8; 32], nonce_prefix: [u8; 4]) -> Self {
        Self {
            cipher: ChaCha20Poly1305::new((&key).into()),
            nonce_prefix,
            tx_counter: 0,
            rx_window: ReplayWindow::default(),
        }
    }

    fn nonce(&self, counter: u64) -> [u8; 12] {
        let mut nonce = [0u8; 12];
        nonce[..4].copy_from_slice(&self.nonce_prefix);
        nonce[4..].copy_from_slice(&counter.to_be_bytes());
        nonce
    }

    /// Returns nonce || ciphertext. The nonce is 12 bytes and is authenticated
    /// as part of the packet framing through the fixed protocol AAD and prefix check.
    pub fn seal(&mut self, plaintext: &[u8]) -> Result<Vec<u8>, SecurityError> {
        let counter = self.tx_counter;
        self.tx_counter = self.tx_counter.checked_add(1).ok_or(SecurityError::CounterExhausted)?;
        let nonce_bytes = self.nonce(counter);
        let ciphertext = self.cipher.encrypt(
            Nonce::from_slice(&nonce_bytes),
            Payload { msg: plaintext, aad: AAD },
        ).map_err(|_| SecurityError::AuthenticationFailed)?;
        let mut packet = Vec::with_capacity(12 + ciphertext.len());
        packet.extend_from_slice(&nonce_bytes);
        packet.extend_from_slice(&ciphertext);
        Ok(packet)
    }

    pub fn open(&mut self, packet: &[u8]) -> Result<Vec<u8>, SecurityError> {
        if packet.len() < 12 + 16 {
            return Err(SecurityError::MalformedPacket);
        }
        if packet[..4] != self.nonce_prefix {
            return Err(SecurityError::WrongSession);
        }
        let counter = u64::from_be_bytes(packet[4..12].try_into().map_err(|_| SecurityError::MalformedPacket)?);
        if !self.rx_window.would_accept(counter) {
            return Err(SecurityError::Replay);
        }
        let plaintext = self.cipher.decrypt(
            Nonce::from_slice(&packet[..12]),
            Payload { msg: &packet[12..], aad: AAD },
        ).map_err(|_| SecurityError::AuthenticationFailed)?;
        // Only authenticated packets advance the replay window.
        self.rx_window.commit(counter);
        Ok(plaintext)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn signed_peer_statement_verifies_and_detects_mutation() {
        let signing = SigningKey::from_bytes(&[7u8; 32]);
        let verifying = signing.verifying_key();
        let mut statement = SignedPeerStatement::sign("node-a".into(), 4, 500, &signing).unwrap();
        assert!(statement.verify(&verifying, 100).is_ok());
        statement.sequence += 1;
        assert_eq!(statement.verify(&verifying, 100), Err(SecurityError::InvalidSignature));
    }

    #[test]
    fn encrypted_packet_round_trip_and_replay_rejection() {
        let key = [9u8; 32];
        let prefix = [1u8, 2, 3, 4];
        let mut tx = SessionCipher::new(key, prefix);
        let mut rx = SessionCipher::new(key, prefix);
        let packet = tx.seal(b"local mesh message").unwrap();
        assert_eq!(rx.open(&packet).unwrap(), b"local mesh message");
        assert_eq!(rx.open(&packet), Err(SecurityError::Replay));
    }

    #[test]
    fn tampering_and_wrong_prefix_are_rejected() {
        let key = [8u8; 32];
        let mut tx = SessionCipher::new(key, [1, 1, 1, 1]);
        let mut rx = SessionCipher::new(key, [1, 1, 1, 1]);
        let mut packet = tx.seal(b"payload").unwrap();
        *packet.last_mut().unwrap() ^= 0x80;
        assert_eq!(rx.open(&packet), Err(SecurityError::AuthenticationFailed));
        let mut other = SessionCipher::new(key, [2, 2, 2, 2]);
        assert_eq!(other.open(&packet), Err(SecurityError::WrongSession));
    }

    #[test]
    fn expired_peer_statement_is_rejected() {
        let signing = SigningKey::from_bytes(&[3u8; 32]);
        let statement = SignedPeerStatement::sign("node-b".into(), 1, 10, &signing).unwrap();
        assert_eq!(statement.verify(&signing.verifying_key(), 10), Err(SecurityError::ExpiredStatement));
    }
}
