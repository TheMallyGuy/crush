// https://github.com/ThreatFlux/FluxEncrypt

use fluxencrypt::SymmetricCipher;
use keyring::Entry;
use std::sync::Mutex;

pub struct CryptoState {
    entry: Entry,
    cipher: Mutex<Option<SymmetricCipher>>,
}

impl CryptoState {
    pub fn new(service: &str, user_id: &str) -> keyring::Result<Self> {
        let entry = Entry::new(service, user_id)?;
        Ok(Self {
            entry,
            cipher: Mutex::new(None),
        })
    }

    fn get_or_create_key(&self) -> anyhow::Result<String> {
        match self.entry.get_password() {
            Ok(key) => Ok(key),
            Err(keyring::Error::NoEntry) => {
                let key = SymmetricCipher::generate_key()?;
                self.entry.set_password(&key)?;
                Ok(key)
            }
            Err(e) => Err(e.into()),
        }
    }

    fn with_cipher<T>(
        &self,
        f: impl FnOnce(&SymmetricCipher) -> anyhow::Result<T>,
    ) -> anyhow::Result<T> {
        let mut guard = self.cipher.lock().unwrap();
        if guard.is_none() {
            let key = self.get_or_create_key()?;
            *guard = Some(SymmetricCipher::new(&key)?);
        }
        f(guard.as_ref().unwrap())
    }

    pub fn encrypt(&self, plaintext: &str) -> anyhow::Result<String> {
        self.with_cipher(|c| c.encrypt(plaintext).map_err(Into::into))
    }

    pub fn decrypt(&self, ciphertext: &str) -> anyhow::Result<String> {
        self.with_cipher(|c| c.decrypt(ciphertext).map_err(Into::into))
    }
}
