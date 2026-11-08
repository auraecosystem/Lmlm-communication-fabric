//! Mutually authenticated ephemeral X25519 handshake.
//!
//! Both long-term Ed25519 public keys must already be trusted by the caller.
//! The handshake binds identities, ephemeral keys and fresh nonces into signed
//! transcripts, then derives separate directional AEAD keys with HKDF-SHA256.
use chacha20poly1305::{aead::{Aead, Payload}, ChaCha20Poly1305, KeyInit, Nonce};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use hkdf::Hkdf;
use rand_core::{OsRng, RngCore};
use sha2::Sha256;
use x25519_dalek::{PublicKey, StaticSecret};

const HELLO_DOMAIN: &[u8] = b"LMLM-LCF/1 handshake hello";
const RESPONSE_DOMAIN: &[u8] = b"LMLM-LCF/1 handshake response";
const KDF_DOMAIN: &[u8] = b"LMLM-LCF/1 directional session keys";
const MAX_ID: usize = 128;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandshakeError {
    InvalidIdentity,
    IdentityMismatch,
    InvalidSignature,
    KeyDerivationFailed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandshakeHello {
    pub initiator_id: String,
    pub responder_id: String,
    pub ephemeral_public: [u8; 32],
    pub nonce: [u8; 32],
    pub signature: [u8; 64],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HandshakeResponse {
    pub initiator_id: String,
    pub responder_id: String,
    pub initiator_ephemeral: [u8; 32],
    pub responder_ephemeral: [u8; 32],
    pub initiator_nonce: [u8; 32],
    pub responder_nonce: [u8; 32],
    pub signature: [u8; 64],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionMaterial {
    pub send_key: [u8; 32],
    pub receive_key: [u8; 32],
    pub send_nonce_prefix: [u8; 4],
    pub receive_nonce_prefix: [u8; 4],
}

impl SessionMaterial {
    pub fn initiator_cipher(&self) -> ChaCha20Poly1305 {
        ChaCha20Poly1305::new_from_slice(&self.send_key).expect("32-byte key")
    }
    pub fn responder_material(&self) -> Self {
        Self {
            send_key: self.receive_key,
            receive_key: self.send_key,
            send_nonce_prefix: self.receive_nonce_prefix,
            receive_nonce_prefix: self.send_nonce_prefix,
        }
    }
}

fn append_id(out: &mut Vec<u8>, id: &str) -> Result<(), HandshakeError> {
    if id.is_empty() || id.len() > MAX_ID { return Err(HandshakeError::InvalidIdentity); }
    out.extend_from_slice(&(id.len() as u16).to_be_bytes());
    out.extend_from_slice(id.as_bytes());
    Ok(())
}

fn hello_bytes(hello: &HandshakeHello) -> Result<Vec<u8>, HandshakeError> {
    let mut out = HELLO_DOMAIN.to_vec();
    append_id(&mut out, &hello.initiator_id)?;
    append_id(&mut out, &hello.responder_id)?;
    out.extend_from_slice(&hello.ephemeral_public);
    out.extend_from_slice(&hello.nonce);
    Ok(out)
}

fn response_bytes(response: &HandshakeResponse) -> Result<Vec<u8>, HandshakeError> {
    let mut out = RESPONSE_DOMAIN.to_vec();
    append_id(&mut out, &response.initiator_id)?;
    append_id(&mut out, &response.responder_id)?;
    out.extend_from_slice(&response.initiator_ephemeral);
    out.extend_from_slice(&response.responder_ephemeral);
    out.extend_from_slice(&response.initiator_nonce);
    out.extend_from_slice(&response.responder_nonce);
    Ok(out)
}

fn derive_material(
    shared: &[u8; 32],
    initiator_id: &str,
    responder_id: &str,
    initiator_ephemeral: &[u8; 32],
    responder_ephemeral: &[u8; 32],
    initiator_nonce: &[u8; 32],
    responder_nonce: &[u8; 32],
    initiator_role: bool,
) -> Result<SessionMaterial, HandshakeError> {
    let mut salt = [0u8; 64];
    salt[..32].copy_from_slice(initiator_nonce);
    salt[32..].copy_from_slice(responder_nonce);
    let mut info = KDF_DOMAIN.to_vec();
    append_id(&mut info, initiator_id)?;
    append_id(&mut info, responder_id)?;
    info.extend_from_slice(initiator_ephemeral);
    info.extend_from_slice(responder_ephemeral);
    let hk = Hkdf::<Sha256>::new(Some(&salt), shared);
    let mut output = [0u8; 72];
    hk.expand(&info, &mut output).map_err(|_| HandshakeError::KeyDerivationFailed)?;
    let mut initiator_key = [0u8; 32];
    let mut responder_key = [0u8; 32];
    let mut initiator_prefix = [0u8; 4];
    let mut responder_prefix = [0u8; 4];
    initiator_key.copy_from_slice(&output[..32]);
    responder_key.copy_from_slice(&output[32..64]);
    initiator_prefix.copy_from_slice(&output[64..68]);
    responder_prefix.copy_from_slice(&output[68..72]);
    if initiator_role {
        Ok(SessionMaterial { send_key: initiator_key, receive_key: responder_key, send_nonce_prefix: initiator_prefix, receive_nonce_prefix: responder_prefix })
    } else {
        Ok(SessionMaterial { send_key: responder_key, receive_key: initiator_key, send_nonce_prefix: responder_prefix, receive_nonce_prefix: initiator_prefix })
    }
}

pub struct HandshakeInitiator {
    local_id: String,
    peer_id: String,
    secret: StaticSecret,
    hello: HandshakeHello,
}

impl HandshakeInitiator {
    pub fn start(local_id: String, peer_id: String, signing_key: &SigningKey) -> Result<(Self, HandshakeHello), HandshakeError> {
        let mut rng = OsRng;
        let secret = StaticSecret::random_from_rng(&mut rng);
        let ephemeral_public = PublicKey::from(&secret).to_bytes();
        let mut nonce = [0u8; 32];
        rng.fill_bytes(&mut nonce);
        let mut hello = HandshakeHello {
            initiator_id: local_id.clone(),
            responder_id: peer_id.clone(),
            ephemeral_public,
            nonce,
            signature: [0u8; 64],
        };
        hello.signature = signing_key.sign(&hello_bytes(&hello)?).to_bytes();
        Ok((Self { local_id, peer_id, secret, hello: hello.clone() }, hello))
    }

    pub fn finish(self, response: &HandshakeResponse, responder_key: &VerifyingKey) -> Result<SessionMaterial, HandshakeError> {
        if response.initiator_id != self.local_id || response.responder_id != self.peer_id
            || response.initiator_ephemeral != self.hello.ephemeral_public
            || response.initiator_nonce != self.hello.nonce {
            return Err(HandshakeError::IdentityMismatch);
        }
        responder_key.verify(&response_bytes(response)?, &Signature::from_bytes(&response.signature))
            .map_err(|_| HandshakeError::InvalidSignature)?;
        let peer_public = PublicKey::from(response.responder_ephemeral);
        let shared = self.secret.diffie_hellman(&peer_public);
        derive_material(
            shared.as_bytes(), &self.local_id, &self.peer_id,
            &self.hello.ephemeral_public, &response.responder_ephemeral,
            &self.hello.nonce, &response.responder_nonce, true,
        )
    }
}

pub fn accept_hello(
    hello: &HandshakeHello,
    local_id: String,
    signing_key: &SigningKey,
    initiator_key: &VerifyingKey,
) -> Result<(HandshakeResponse, SessionMaterial), HandshakeError> {
    if hello.responder_id != local_id { return Err(HandshakeError::IdentityMismatch); }
    initiator_key.verify(&hello_bytes(hello)?, &Signature::from_bytes(&hello.signature))
        .map_err(|_| HandshakeError::InvalidSignature)?;
    let mut rng = OsRng;
    let secret = StaticSecret::random_from_rng(&mut rng);
    let responder_ephemeral = PublicKey::from(&secret).to_bytes();
    let mut responder_nonce = [0u8; 32];
    rng.fill_bytes(&mut responder_nonce);
    let mut response = HandshakeResponse {
        initiator_id: hello.initiator_id.clone(),
        responder_id: local_id.clone(),
        initiator_ephemeral: hello.ephemeral_public,
        responder_ephemeral,
        initiator_nonce: hello.nonce,
        responder_nonce,
        signature: [0u8; 64],
    };
    response.signature = signing_key.sign(&response_bytes(&response)?).to_bytes();
    let peer_public = PublicKey::from(hello.ephemeral_public);
    let shared = secret.diffie_hellman(&peer_public);
    let material = derive_material(
        shared.as_bytes(), &hello.initiator_id, &local_id,
        &hello.ephemeral_public, &responder_ephemeral,
        &hello.nonce, &responder_nonce, false,
    )?;
    Ok((response, material))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mutual_handshake_derives_matching_directional_keys() {
        let alice = SigningKey::from_bytes(&[11u8; 32]);
        let bob = SigningKey::from_bytes(&[22u8; 32]);
        let (initiator, hello) = HandshakeInitiator::start("alice".into(), "bob".into(), &alice).unwrap();
        let (response, bob_material) = accept_hello(&hello, "bob".into(), &bob, &alice.verifying_key()).unwrap();
        let alice_material = initiator.finish(&response, &bob.verifying_key()).unwrap();
        assert_eq!(alice_material.send_key, bob_material.receive_key);
        assert_eq!(alice_material.receive_key, bob_material.send_key);
        assert_eq!(alice_material.send_nonce_prefix, bob_material.receive_nonce_prefix);
        assert_eq!(alice_material.receive_nonce_prefix, bob_material.send_nonce_prefix);
    }

    #[test]
    fn handshake_rejects_tampered_identity_binding() {
        let alice = SigningKey::from_bytes(&[11u8; 32]);
        let bob = SigningKey::from_bytes(&[22u8; 32]);
        let (initiator, hello) = HandshakeInitiator::start("alice".into(), "bob".into(), &alice).unwrap();
        let (mut response, _) = accept_hello(&hello, "bob".into(), &bob, &alice.verifying_key()).unwrap();
        response.responder_id = "mallory".into();
        assert_eq!(initiator.finish(&response, &bob.verifying_key()), Err(HandshakeError::IdentityMismatch));
    }
}
