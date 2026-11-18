//! Installs an update of an installed Homebrew app that Luma downloaded from Homebrew Channel's
//! repository (opt-in setting). Everything Luma checked is checked again here as root: the app is
//! a Homebrew installation, it is not running, and the package matches its repository checksum.
use crate::{
    APP, OTHER_LUMA_APPS, Result, accepted, ipk,
    system::{self, System},
    valid_app_id,
};
use serde_json::{Value, json};
use std::{fs, path::Path};

/// Homebrew Channel installs apps here, as developer-mode packages.
pub const APPS: &str = "/media/developer/apps/usr/palm/applications";
pub fn excluded(id: &str) -> bool {
    id == APP
        || matches!(
            id,
            "org.webosbrew.hbchannel" | "org.webosbrew.hbchannel.service"
        )
        || OTHER_LUMA_APPS.contains(&id)
        || ["com.webos.", "com.palm.", "com.lge."]
            .iter()
            .any(|prefix| id.starts_with(prefix))
}
const STATE: &str = "/var/lib/luma-evolution-update";
const WORK: &str = "/var/lib/luma-evolution-update/apps";

/// Shipping releases use strict three-part numeric versions, never lexical comparison.
pub fn numeric_version(value: &str) -> Option<[u64; 3]> {
    if value.len() > 32 {
        return None;
    }
    let parts: Vec<_> = value.split('.').collect();
    if parts.len() != 3 {
        return None;
    }
    let mut result = [0; 3];
    for (index, part) in parts.into_iter().enumerate() {
        if part.is_empty()
            || (part.len() > 1 && part.starts_with('0'))
            || !part.bytes().all(|byte| byte.is_ascii_digit())
        {
            return None;
        }
        result[index] = part.parse().ok()?;
    }
    Some(result)
}

fn valid_sha256(sha: &str) -> bool {
    sha.len() == 64
        && sha
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// The version of an installed Homebrew app, or an error if it is not one.
pub fn installed_version(system: &impl System, id: &str) -> Result<String> {
    let manifest = system::json(system, format!("{APPS}/{id}/appinfo.json"))?;
    if manifest["id"] != id {
        return Err("Not an installed Homebrew app".into());
    }
    manifest["version"]
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| "Installed app has no version".into())
}

fn running(system: &impl System, id: &str) -> Result<bool> {
    let reply = accepted(system::luna(
        system,
        "com.webos.applicationManager",
        "running",
        &json!({}),
    )?)?;
    Ok(reply["running"]
        .as_array()
        .is_some_and(|apps| apps.iter().any(|app| app["id"] == id)))
}

fn sha256(bytes: &[u8]) -> String {
    ring::digest::digest(&ring::digest::SHA256, bytes)
        .as_ref()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

#[cfg(unix)]
fn private_directory(system: &impl System, path: &str) -> Result<()> {
    use std::os::unix::fs::{DirBuilderExt, MetadataExt, PermissionsExt};

    let path = system.path(Path::new(path));
    match fs::DirBuilder::new().mode(0o700).create(&path) {
        Ok(()) => return Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error.to_string()),
    }
    let metadata = fs::symlink_metadata(&path).map_err(|error| error.to_string())?;
    if !metadata.is_dir() || metadata.uid() != system::Tv.uid() || metadata.mode() & 0o022 != 0 {
        return Err("Unsafe IPK staging directory".into());
    }
    fs::set_permissions(path, fs::Permissions::from_mode(0o700)).map_err(|error| error.to_string())
}

#[cfg(not(unix))]
fn private_directory(_system: &impl System, _path: &str) -> Result<()> {
    Err("IPK staging requires Unix".into())
}

/// Installs `ipk` as `version` of the Homebrew app `id`. The package is first copied into a
/// root-owned directory, so nothing can change it between the checksum check and LG's installer.
pub fn install(
    system: &impl System,
    id: &str,
    version: &str,
    sha: &str,
    ipk: &Path,
) -> Result<Value> {
    system::root(system)?;
    if !valid_app_id(id) || excluded(id) {
        return Err("This app is not updated by Luma".into());
    }
    let Some(candidate) = numeric_version(version) else {
        return Err("Invalid update version".into());
    };
    if !valid_sha256(sha) || !ipk.is_absolute() {
        return Err("Invalid update request".into());
    }
    let current = installed_version(system, id)?;
    if current == version {
        return Ok(json!({"returnValue": true, "version": version, "changed": false}));
    }
    if running(system, id)? {
        return Err("The app is running; the update waits until it is closed".into());
    }
    let bytes = system::read(system, ipk, ipk::MAX_IPK)?;
    if sha256(&bytes) != sha {
        return Err("The package does not match its repository checksum".into());
    }
    let architecture = ipk::verify(&bytes, id, version)?;
    private_directory(system, STATE)?;
    private_directory(system, WORK)?;
    let lock = fs::OpenOptions::new()
        .create(true)
        .truncate(false)
        .write(true)
        .open(system.path(&Path::new(WORK).join(format!("{id}.lock"))))
        .map_err(|e| e.to_string())?;
    if !system::lock(&lock) {
        return Err("An update for this app is already running".into());
    }
    let current = installed_version(system, id)?;
    if current == version {
        return Ok(json!({"returnValue": true, "version": version, "changed": false}));
    }
    if numeric_version(&current).is_none_or(|current| current >= candidate) {
        return Err("The requested version is older than or equal to the installed version".into());
    }
    let copy = Path::new(WORK).join(format!("{id}_{version}_{architecture}.ipk"));
    let result = (|| {
        system::write(system, &copy, &bytes, 0o600)?;
        // Checked once more right before installing: the user may have opened the app meanwhile.
        if running(system, id)? {
            return Err("The app is running; the update waits until it is closed".into());
        }
        accepted(system::luna(
            system,
            "com.webos.appInstallService",
            "dev/install",
            &json!({"id": id, "ipkUrl": copy.to_string_lossy(), "subscribe": false}),
        )?)?;
        // LG's installer answers before it finishes; the installed manifest is the proof.
        for _ in 0..240 {
            if installed_version(system, id).is_ok_and(|v| v == version) {
                return Ok(());
            }
            system.pause();
        }
        Err(format!("{id} was not updated to {version} in time"))
    })();
    let _ = system::remove(system, &copy);
    result?;
    Ok(json!({"returnValue": true, "version": version, "changed": true}))
}
