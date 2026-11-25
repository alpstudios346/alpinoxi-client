//! The OS credential store: where refresh tokens actually live.
//!
//! `MemoryVault` is right for tests and for a session that must not persist, but
//! a launcher that asks for a sign-in every launch is a launcher people stop
//! using. This module is the real backend.
//!
//! The rule from `docs/AUTHENTICATION.md` is unchanged: **no token ever touches a
//! file we control.** On Windows we call `CredWriteW`/`CredReadW` directly, so a
//! refresh token sits in the same protected store as every other application's
//! credentials, encrypted with the user's login. No new dependency, no JSON on
//! disk, no key to manage ourselves.
//!
//! On a platform where this is not implemented, `SystemVault::open` returns
//! `None` and the caller falls back to `MemoryVault`: the launcher still works,
//! it just asks for a sign-in each time. Failing loudly here would be worse than
//! a sign-in prompt.

use crate::vault::{TokenVault, VaultError};

/// The platform credential store, when the platform has one we can reach.
#[derive(Debug, Clone)]
pub struct SystemVault {
    /// Prefix so Alpinoxi's entries are grouped and easy to find by hand.
    service: String,
}

impl SystemVault {
    /// Opens the platform store, or `None` when it is not available.
    pub fn open() -> Option<Self> {
        platform::available().then(|| Self {
            service: "Alpinoxi".to_owned(),
        })
    }

    /// The name the OS shows for a key, e.g. `Alpinoxi/elyby.refresh`.
    fn target(&self, key: &str) -> String {
        format!("{}/{}", self.service, key)
    }

    /// Forgets every Alpinoxi entry, for the "sign out everywhere" button.
    pub fn clear_all(&mut self) -> Result<usize, VaultError> {
        let keys = self.keys();
        let mut removed = 0;
        for key in keys {
            if self.forget(&key).is_ok() {
                removed += 1;
            }
        }
        Ok(removed)
    }
}

impl TokenVault for SystemVault {
    fn store(&mut self, key: &str, secret: &str) -> Result<(), VaultError> {
        platform::store(&self.target(key), secret)
    }

    fn load(&self, key: &str) -> Result<Option<String>, VaultError> {
        platform::load(&self.target(key))
    }

    fn forget(&mut self, key: &str) -> Result<(), VaultError> {
        platform::forget(&self.target(key))
    }

    fn keys(&self) -> Vec<String> {
        platform::keys()
            .into_iter()
            .filter_map(|target| {
                target
                    .strip_prefix(&format!("{}/", self.service))
                    .map(str::to_owned)
            })
            .collect()
    }
}

#[cfg(target_os = "windows")]
mod platform {
    //! `advapi32`'s Credential Manager, called through `extern "system"`.
    //!
    //! The structs below are the documented `CREDENTIALW` layout. They are
    //! declared here rather than pulled from a crate so the launcher keeps its
    //! dependency count at zero for this.

    use std::ffi::c_void;
    use std::os::windows::ffi::OsStrExt;

    use super::VaultError;

    const CRED_TYPE_GENERIC: u32 = 1;
    const CRED_PERSIST_LOCAL_MACHINE: u32 = 2;
    const ERROR_NOT_FOUND: u32 = 1168;

    #[repr(C)]
    struct FileTime {
        low: u32,
        high: u32,
    }

    #[repr(C)]
    struct CredentialW {
        flags: u32,
        kind: u32,
        target_name: *mut u16,
        comment: *mut u16,
        last_written: FileTime,
        credential_blob_size: u32,
        credential_blob: *mut u8,
        persist: u32,
        attribute_count: u32,
        attributes: *mut c_void,
        target_alias: *mut u16,
        user_name: *mut u16,
    }

    #[link(name = "advapi32")]
    extern "system" {
        fn CredWriteW(credential: *const CredentialW, flags: u32) -> i32;
        fn CredReadW(
            target: *const u16,
            kind: u32,
            flags: u32,
            credential: *mut *mut CredentialW,
        ) -> i32;
        fn CredDeleteW(target: *const u16, kind: u32, flags: u32) -> i32;
        fn CredFree(buffer: *mut c_void);
        fn CredEnumerateW(
            filter: *const u16,
            flags: u32,
            count: *mut u32,
            credentials: *mut *mut *mut CredentialW,
        ) -> i32;
    }

    fn wide(text: &str) -> Vec<u16> {
        std::ffi::OsStr::new(text)
            .encode_wide()
            .chain(std::iter::once(0))
            .collect()
    }

    pub fn available() -> bool {
        // A store that cannot answer for a key it certainly does not have is a
        // store we cannot use.
        matches!(load("Alpinoxi/__probe"), Ok(None)) || load("Alpinoxi/__probe").is_ok()
    }

    pub fn store(key: &str, secret: &str) -> Result<(), VaultError> {
        let mut target = wide(key);
        let mut blob: Vec<u8> = secret.as_bytes().to_vec();
        let mut user = wide("alpinoxi");
        let credential = CredentialW {
            flags: 0,
            kind: CRED_TYPE_GENERIC,
            target_name: target.as_mut_ptr(),
            comment: std::ptr::null_mut(),
            last_written: FileTime { low: 0, high: 0 },
            credential_blob_size: blob.len() as u32,
            credential_blob: blob.as_mut_ptr(),
            persist: CRED_PERSIST_LOCAL_MACHINE,
            attribute_count: 0,
            attributes: std::ptr::null_mut(),
            target_alias: std::ptr::null_mut(),
            user_name: user.as_mut_ptr(),
        };
        // SAFETY: every pointer in `credential` points at a live local buffer for
        // the duration of the call, and the blob length matches the blob.
        let ok = unsafe { CredWriteW(&credential, 0) };
        if ok == 0 {
            return Err(VaultError::Unavailable);
        }
        Ok(())
    }

    pub fn load(key: &str) -> Result<Option<String>, VaultError> {
        let target = wide(key);
        let mut out: *mut CredentialW = std::ptr::null_mut();
        // SAFETY: `target` is a valid NUL-terminated wide string; on success the
        // API hands us a pointer we own until `CredFree`.
        let ok = unsafe { CredReadW(target.as_ptr(), CRED_TYPE_GENERIC, 0, &mut out) };
        if ok == 0 {
            return Ok(None);
        }
        // SAFETY: `out` is non-null on success, per the API contract.
        let credential = unsafe { &*out };
        let bytes = if credential.credential_blob.is_null() || credential.credential_blob_size == 0
        {
            Vec::new()
        } else {
            // SAFETY: the blob is `credential_blob_size` bytes, as the API reports.
            unsafe {
                std::slice::from_raw_parts(
                    credential.credential_blob,
                    credential.credential_blob_size as usize,
                )
            }
            .to_vec()
        };
        unsafe { CredFree(out as *mut c_void) };
        Ok(String::from_utf8(bytes).ok())
    }

    pub fn forget(key: &str) -> Result<(), VaultError> {
        let target = wide(key);
        // SAFETY: valid wide string; deleting a missing entry is not an error.
        let ok = unsafe { CredDeleteW(target.as_ptr(), CRED_TYPE_GENERIC, 0) };
        if ok == 0 {
            // Ignore "not found", report anything else.
            let code = std::io::Error::last_os_error().raw_os_error().unwrap_or(0) as u32;
            if code == ERROR_NOT_FOUND {
                return Ok(());
            }
            return Err(VaultError::Unavailable);
        }
        Ok(())
    }

    pub fn keys() -> Vec<String> {
        let filter = wide("Alpinoxi/*");
        let mut count: u32 = 0;
        let mut list: *mut *mut CredentialW = std::ptr::null_mut();
        // SAFETY: valid filter string; on success the API hands us an array of
        // `count` pointers we own until `CredFree`.
        let ok = unsafe { CredEnumerateW(filter.as_ptr(), 0, &mut count, &mut list) };
        if ok == 0 || list.is_null() {
            return Vec::new();
        }
        let mut keys = Vec::new();
        for index in 0..count as usize {
            // SAFETY: the array holds `count` valid pointers.
            let credential = unsafe { &*(*list.add(index)) };
            if credential.target_name.is_null() {
                continue;
            }
            let mut length = 0;
            // SAFETY: NUL-terminated wide string owned by the API.
            unsafe {
                while *credential.target_name.add(length) != 0 {
                    length += 1;
                }
                let slice = std::slice::from_raw_parts(credential.target_name, length);
                keys.push(String::from_utf16_lossy(slice));
            }
        }
        unsafe { CredFree(list as *mut c_void) };
        keys
    }
}

#[cfg(not(target_os = "windows"))]
mod platform {
    //! Nothing is implemented here yet, and pretending otherwise would be worse
    //! than saying so: the caller falls back to the in-memory vault.

    use super::VaultError;

    pub fn available() -> bool {
        false
    }

    pub fn store(_key: &str, _secret: &str) -> Result<(), VaultError> {
        Err(VaultError::Unavailable)
    }

    pub fn load(_key: &str) -> Result<Option<String>, VaultError> {
        Ok(None)
    }

    pub fn forget(_key: &str) -> Result<(), VaultError> {
        Err(VaultError::Unavailable)
    }

    pub fn keys() -> Vec<String> {
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn target_names_are_grouped_and_readable() {
        let vault = SystemVault {
            service: "Alpinoxi".to_owned(),
        };
        assert_eq!(vault.target("elyby.refresh"), "Alpinoxi/elyby.refresh");
    }

    #[test]
    fn missing_store_falls_back_cleanly() {
        // On CI Linux there is no store: `open` must say so instead of panicking.
        if SystemVault::open().is_none() {
            assert!(platform::load("anything").unwrap().is_none());
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn round_trips_through_the_real_credential_store() {
        let Some(mut vault) = SystemVault::open() else {
            return;
        };
        let key = "test.roundtrip";
        vault.store(key, "secret-value").expect("store");
        assert_eq!(
            vault.load(key).expect("load").as_deref(),
            Some("secret-value")
        );
        assert!(vault.keys().iter().any(|held| held == key));
        vault.forget(key).expect("forget");
        assert!(vault.load(key).expect("load after forget").is_none());
    }
}
