use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use stealth_bonding_sim::{
    generate_credential_id, generate_identity, generate_secret, public_key, PublicKeyBytes, Secret,
    DEFAULT_MANUFACTURER_SECRET,
};

const MAGIC: &str = "SBVAULT1";

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FileVault {
    path: PathBuf,
    pub identity: [u8; 6],
    pub secret: Secret,
    pub credential_id: [u8; 16],
    pub manufacturer_root: PublicKeyBytes,
    used_nonces: Vec<[u8; 32]>,
}

impl FileVault {
    pub fn create(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        if path.exists() {
            return Err(io::Error::new(
                ErrorKind::AlreadyExists,
                "vault file already exists",
            ));
        }
        let vault = Self {
            path,
            identity: generate_identity(),
            secret: generate_secret(),
            credential_id: generate_credential_id(),
            manufacturer_root: public_key(&DEFAULT_MANUFACTURER_SECRET),
            used_nonces: Vec::new(),
        };
        vault.save()?;
        Ok(vault)
    }

    pub fn load(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let text = fs::read_to_string(&path)?;
        Self::parse(path, &text)
    }

    pub fn create_or_load(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();
        if path.exists() {
            Self::load(path)
        } else {
            Self::create(path)
        }
    }

    pub fn public_key(&self) -> PublicKeyBytes {
        public_key(&self.secret)
    }

    pub fn remember_nonce(&mut self, nonce: [u8; 32]) -> io::Result<bool> {
        if self.used_nonces.contains(&nonce) {
            return Ok(false);
        }
        self.used_nonces.push(nonce);
        self.save()?;
        Ok(true)
    }

    fn save(&self) -> io::Result<()> {
        if let Some(parent) = self.path.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent)?;
            }
        }
        fs::write(&self.path, self.encode())?;
        restrict_permissions(&self.path)?;
        Ok(())
    }

    fn encode(&self) -> String {
        let mut out = String::from(MAGIC);
        out.push('\n');
        out.push_str(&format!("identity {}\n", hex(&self.identity)));
        out.push_str(&format!("credential_id {}\n", hex(&self.credential_id)));
        out.push_str(&format!("secret {}\n", hex(&self.secret)));
        out.push_str(&format!("manufacturer {}\n", hex(&self.manufacturer_root)));
        for nonce in &self.used_nonces {
            out.push_str(&format!("used_nonce {}\n", hex(nonce)));
        }
        out
    }

    fn parse(path: PathBuf, text: &str) -> io::Result<Self> {
        let mut lines = text.lines();
        let magic = lines.next().unwrap_or_default();
        if magic != MAGIC {
            return Err(invalid("vault magic is not SBVAULT1"));
        }
        let mut identity = None;
        let mut credential_id = None;
        let mut secret = None;
        let mut manufacturer_root = None;
        let mut used_nonces = Vec::new();
        for line in lines {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let (key, value) = line
                .split_once(' ')
                .ok_or_else(|| invalid("vault line is missing a value"))?;
            match key {
                "identity" => identity = Some(parse_hex_array::<6>(value)?),
                "credential_id" => credential_id = Some(parse_hex_array::<16>(value)?),
                "secret" => secret = Some(parse_hex_array::<32>(value)?),
                "manufacturer" => manufacturer_root = Some(parse_hex_array::<33>(value)?),
                "used_nonce" => used_nonces.push(parse_hex_array::<32>(value)?),
                _ => return Err(invalid("unknown vault field")),
            }
        }
        Ok(Self {
            path,
            identity: identity.ok_or_else(|| invalid("vault is missing identity"))?,
            secret: secret.ok_or_else(|| invalid("vault is missing secret"))?,
            credential_id: credential_id
                .ok_or_else(|| invalid("vault is missing credential_id"))?,
            manufacturer_root: manufacturer_root
                .ok_or_else(|| invalid("vault is missing manufacturer root"))?,
            used_nonces,
        })
    }
}

fn restrict_permissions(path: &Path) -> io::Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    let _ = path;
    Ok(())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn parse_hex_array<const N: usize>(value: &str) -> io::Result<[u8; N]> {
    if value.len() != N * 2 {
        return Err(invalid("vault hex field has the wrong length"));
    }
    let mut out = [0u8; N];
    for (index, chunk) in value.as_bytes().chunks(2).enumerate() {
        let text = std::str::from_utf8(chunk).map_err(|_| invalid("vault hex is not UTF-8"))?;
        out[index] =
            u8::from_str_radix(text, 16).map_err(|_| invalid("vault hex is not hexadecimal"))?;
    }
    Ok(out)
}

fn invalid(message: &str) -> io::Error {
    io::Error::new(ErrorKind::InvalidData, message)
}
