//! Validate the archive LG will receive without extracting it or interpreting tar extensions.
use crate::{APP, Result, app_update::numeric_version, valid_app_id};
use flate2::bufread::GzDecoder;
use serde::{
    Deserialize,
    de::{Error, IgnoredAny, MapAccess, Visitor},
};
use std::{
    collections::{BTreeMap, BTreeSet},
    io::{self, Read},
};

pub const MAX_IPK: usize = 96 * 1024 * 1024;
const MAX_CONTROL: u64 = 256 * 1024;
const MAX_DATA: u64 = 384 * 1024 * 1024;
const MAX_MANIFEST: u64 = 1024 * 1024;

fn ar_member<'a>(bytes: &mut &'a [u8], name: &str) -> Option<&'a [u8]> {
    let (header, remaining) = bytes.split_at_checked(60)?;
    let actual = std::str::from_utf8(&header[..16])
        .ok()?
        .trim_end_matches(' ');
    let length = std::str::from_utf8(&header[48..58]).ok()?.trim_matches(' ');
    if actual.strip_suffix('/').unwrap_or(actual) != name
        || &header[58..] != b"\x60\n"
        || !length.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    let size: usize = length.parse().ok()?;
    let (data, mut rest) = remaining.split_at_checked(size)?;
    if !size.is_multiple_of(2) {
        rest = rest.strip_prefix(b"\n")?;
    }
    *bytes = rest;
    Some(data)
}

fn octal(bytes: &[u8]) -> Option<u64> {
    let value = std::str::from_utf8(bytes)
        .ok()?
        .trim_start_matches(' ')
        .trim_end_matches([' ', '\0']);
    if value.is_empty() || !value.bytes().all(|byte| (b'0'..=b'7').contains(&byte)) {
        return None;
    }
    u64::from_str_radix(value, 8).ok()
}

fn field(bytes: &[u8]) -> Option<&str> {
    let end = bytes
        .iter()
        .position(|byte| *byte == 0)
        .unwrap_or(bytes.len());
    zero(&bytes[end..])
        .then(|| std::str::from_utf8(&bytes[..end]).ok())
        .flatten()
}

fn ustar_prefix(header: &[u8; 512]) -> Option<&str> {
    if header[475] != 0 {
        return field(&header[345..500]);
    }
    // ares-package's node-tar stores two octal timestamps after a NUL-separated
    // short prefix. Neither is part of the name seen by a POSIX tar reader.
    for offset in [476, 488] {
        let stamp = &header[offset..offset + 12];
        if !zero(stamp)
            && (stamp[11] != 0 || !stamp[..11].iter().all(|byte| (b'0'..=b'7').contains(byte)))
        {
            return None;
        }
    }
    field(&header[345..475])
}

fn zero(bytes: &[u8]) -> bool {
    bytes.iter().all(|byte| *byte == 0)
}

fn canonical_path(name: &str, directory: bool) -> Option<&str> {
    if directory && matches!(name, "." | "./") {
        return Some("");
    }
    let path = name.strip_prefix("./").unwrap_or(name);
    let path = if directory {
        path.strip_suffix('/').unwrap_or(path)
    } else {
        path
    };
    if path
        .bytes()
        .any(|byte| byte.is_ascii_control() || matches!(byte, b'\\' | b':'))
        || path.split('/').any(|part| matches!(part, "" | "." | ".."))
    {
        return None;
    }
    Some(path)
}

fn archive(
    bytes: &[u8],
    limit: u64,
    mut visit: impl FnMut(&str, bool, u64, &mut dyn Read) -> Result<()>,
) -> Result<()> {
    let invalid = || "Invalid or unsupported IPK tar archive".to_owned();
    if bytes.get(..3) != Some(b"\x1f\x8b\x08") || bytes.get(3).is_none_or(|flags| flags & 0xe0 != 0)
    {
        return Err(invalid());
    }
    let mut decoder = GzDecoder::new(bytes);
    let mut reader = decoder.by_ref().take(limit + 1);
    let mut paths = BTreeMap::new();
    let mut header = [0; 512];
    loop {
        reader.read_exact(&mut header).map_err(|_| invalid())?;
        if zero(&header) {
            let mut blocks = 1;
            loop {
                match reader.read(&mut header[..1]).map_err(|_| invalid())? {
                    0 => break,
                    _ => reader.read_exact(&mut header[1..]).map_err(|_| invalid())?,
                }
                if !zero(&header) {
                    return Err(invalid());
                }
                blocks += 1;
            }
            if blocks < 2 || reader.limit() == 0 || !decoder.get_ref().is_empty() {
                return Err(invalid());
            }
            break;
        }
        let size = octal(&header[124..136]).ok_or_else(invalid)?;
        let mode = octal(&header[100..108]).ok_or_else(invalid)?;
        let checksum: u64 = header
            .iter()
            .enumerate()
            .map(|(index, &byte)| {
                u64::from(if (148..156).contains(&index) {
                    b' '
                } else {
                    byte
                })
            })
            .sum();
        let directory = header[156] == b'5';
        // Packages such as Moonlight include the matching POSIX file type in
        // the mode. Strip only that type when checking for special mode bits.
        let mode_type = mode & !0o777;
        if octal(&header[148..156]) != Some(checksum)
            || (mode_type != 0 && mode_type != if directory { 0o040000 } else { 0o100000 })
            || (!directory && !matches!(header[156], 0 | b'0'))
            || (directory && size != 0)
            || !zero(&header[157..257])
            || paths.len() >= 50_000
        {
            return Err(invalid());
        }
        let prefix = match &header[257..265] {
            b"ustar\0\x30\x30" if zero(&header[500..]) => {
                ustar_prefix(&header).ok_or_else(invalid)?
            }
            b"ustar  \0" if zero(&header[345..]) => "",
            _ => return Err(invalid()),
        };
        let basename = field(&header[..100]).ok_or_else(invalid)?;
        if basename.is_empty() || (!prefix.is_empty() && canonical_path(prefix, false).is_none()) {
            return Err(invalid());
        }
        let combined;
        let name = if prefix.is_empty() {
            basename
        } else {
            combined = format!("{prefix}/{basename}");
            &combined
        };
        let name = canonical_path(name, directory).ok_or_else(invalid)?;
        if paths.insert(name.to_owned(), directory).is_some() {
            return Err(format!("Duplicate IPK tar path: {name}"));
        }
        if size.div_ceil(512) * 512 >= reader.limit() {
            return Err(invalid());
        }
        let mut file = reader.by_ref().take(size);
        visit(name, directory, size, &mut file)?;
        io::copy(&mut file, &mut io::sink()).map_err(|_| invalid())?;
        if file.limit() != 0 {
            return Err(invalid());
        }
        let mut padding = [0; 512];
        let rest = &mut padding[..((512 - size % 512) % 512) as usize];
        reader.read_exact(rest).map_err(|_| invalid())?;
        if !zero(rest) {
            return Err(invalid());
        }
    }
    for path in paths.keys() {
        for (index, _) in path.match_indices('/') {
            if paths.get(&path[..index]) == Some(&false) {
                return Err(format!("Conflicting IPK tar paths: {path}"));
            }
        }
    }
    Ok(())
}

fn control_fields(text: &str) -> Result<BTreeMap<String, &str>> {
    if text.bytes().enumerate().any(|(index, byte)| {
        byte.is_ascii_control()
            && !matches!(byte, b'\t' | b'\n')
            && !(byte == b'\r' && text.as_bytes().get(index + 1) == Some(&b'\n'))
    }) {
        return Err("Invalid IPK control text".into());
    }
    let mut fields = BTreeMap::new();
    let mut previous = String::new();
    for line in text.trim_end().lines() {
        if line.starts_with([' ', '\t']) && previous == "description" {
            continue;
        }
        let (key, value) = line.split_once(':').ok_or("Invalid IPK control field")?;
        let key = key.to_ascii_lowercase();
        let value = value.trim_matches([' ', '\t']);
        if !matches!(
            key.as_str(),
            "package"
                | "version"
                | "architecture"
                | "description"
                | "maintainer"
                | "section"
                | "priority"
                | "installed-size"
                | "webos-package-format-version"
                | "webos-packager-version"
                | "homepage"
                | "source"
                | "license"
        ) || value.bytes().any(|byte| byte.is_ascii_control())
            || fields.insert(key.clone(), value).is_some()
        {
            return Err(format!("Unexpected or duplicate IPK control field: {key}"));
        }
        previous = key;
    }
    Ok(fields)
}

#[derive(Deserialize)]
struct AppInfo {
    id: String,
    version: String,
    #[serde(rename = "type")]
    kind: Option<String>,
    main: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct PackageInfo {
    id: String,
    version: String,
    app: String,
    #[serde(default)]
    services: Vec<String>,
}

#[derive(Deserialize)]
struct ServiceInfo {
    id: Option<String>,
    services: Vec<Service>,
}

#[derive(Deserialize)]
struct Service {
    name: String,
}

struct UniqueManifestKeys;

impl<'de> Visitor<'de> for UniqueManifestKeys {
    type Value = ();

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("a manifest object with unique keys")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> std::result::Result<(), A::Error> {
        let mut keys = BTreeSet::new();
        while let Some(key) = map.next_key::<String>()? {
            if !keys.insert(key) {
                return Err(A::Error::custom("duplicate IPK manifest key"));
            }
            map.next_value::<IgnoredAny>()?;
        }
        Ok(())
    }
}

fn metadata<T: serde::de::DeserializeOwned>(reader: &mut dyn Read, size: u64) -> Result<T> {
    if size > MAX_MANIFEST {
        return Err("IPK metadata exceeds the size limit".into());
    }
    let mut bytes = Vec::with_capacity(size as usize);
    reader
        .read_to_end(&mut bytes)
        .map_err(|error| error.to_string())?;
    serde::Deserializer::deserialize_map(
        &mut serde_json::Deserializer::from_slice(&bytes),
        UniqueManifestKeys,
    )
    .map_err(|error| format!("Invalid IPK manifest: {error}"))?;
    serde_json::from_slice(&bytes).map_err(|error| format!("Invalid IPK manifest: {error}"))
}

fn verify_payload(bytes: &[u8], id: &str, version: &str) -> Result<()> {
    let app = format!("usr/palm/applications/{id}");
    let package = format!("usr/palm/packages/{id}");
    let package_path = format!("{package}/packageinfo.json");
    let mut app_manifest: Option<AppInfo> = None;
    let mut package_manifest: Option<PackageInfo> = None;
    let mut services = BTreeMap::<String, Option<ServiceInfo>>::new();
    let mut scripts = 0;
    let service_id = |name: &str| {
        valid_app_id(name)
            && name
                .strip_prefix(id)
                .is_some_and(|suffix| suffix.starts_with('.'))
    };
    archive(bytes, MAX_DATA, |name, directory, size, reader| {
        if let Some(relative) = name
            .strip_prefix(&app)
            .and_then(|rest| rest.strip_prefix('/'))
        {
            match relative {
                "appinfo.json" if !directory => app_manifest = Some(metadata(reader, size)?),
                "autostart.sh" | "home.sh" if id == APP => {
                    if directory || size > MAX_CONTROL {
                        return Err("Invalid Evolution POSIX script".into());
                    }
                    let mut script = Vec::with_capacity(size as usize);
                    reader
                        .read_to_end(&mut script)
                        .map_err(|error| error.to_string())?;
                    if !script.starts_with(b"#!/bin/sh\n")
                        || script.contains(&b'\r')
                        || script.contains(&0)
                    {
                        return Err("Evolution POSIX scripts must use LF line endings".into());
                    }
                    scripts += 1;
                }
                _ => {}
            }
            return Ok(());
        }
        if name == package_path && !directory {
            package_manifest = Some(metadata(reader, size)?);
            return Ok(());
        }
        if directory
            && (matches!(
                name,
                "" | "usr" | "usr/palm" | "usr/palm/applications" | "usr/palm/packages"
            ) || name == app
                || name == package
                || (id != APP && name == "usr/palm/services"))
        {
            return Ok(());
        }
        if id != APP
            && let Some(path) = name.strip_prefix("usr/palm/services/")
        {
            let (service, relative) = path.split_once('/').unwrap_or((path, ""));
            if service_id(service) && (!relative.is_empty() || directory) {
                let manifest = services.entry(service.to_owned()).or_default();
                if relative == "services.json" && !directory {
                    *manifest = Some(metadata(reader, size)?);
                }
                return Ok(());
            }
        }
        Err(format!("Unexpected IPK payload path: {name}"))
    })?;

    let app_manifest = app_manifest.ok_or("Missing IPK appinfo.json")?;
    if app_manifest.id != id || app_manifest.version != version {
        return Err("IPK app manifest identity or version does not match the request".into());
    }
    if id == APP
        && (app_manifest.kind.as_deref() != Some("native")
            || app_manifest.main.as_deref() != Some("luma")
            || scripts != 2)
    {
        return Err("Invalid native Evolution manifest or missing POSIX scripts".into());
    }
    if let Some(package_manifest) = package_manifest {
        let declared = package_manifest
            .services
            .iter()
            .map(String::as_str)
            .collect::<BTreeSet<_>>();
        if package_manifest.id != id
            || package_manifest.app != id
            || package_manifest.version != version
            || declared.len() != package_manifest.services.len()
            || !declared.into_iter().eq(services.keys().map(String::as_str))
        {
            return Err(
                "IPK package manifest identity, version or services do not match the payload"
                    .into(),
            );
        }
    } else if id == APP || !services.is_empty() {
        return Err("Missing IPK packageinfo.json".into());
    }
    for (directory, info) in services {
        let info = info.ok_or("Missing IPK services.json")?;
        if info.id.as_ref().is_some_and(|name| !service_id(name))
            || !info
                .services
                .iter()
                .any(|service| service.name == directory)
                && info.id.as_deref() != Some(&directory)
            || info
                .services
                .iter()
                .any(|service| !service_id(&service.name))
        {
            return Err("IPK declares a foreign service identity".into());
        }
    }
    Ok(())
}

/// Returns the validated architecture for the installer's staged filename.
pub fn verify(ipk: &[u8], id: &str, version: &str) -> Result<&'static str> {
    let invalid = || "Invalid or incomplete IPK archive".to_owned();
    if ipk.len() > MAX_IPK || !valid_app_id(id) || numeric_version(version).is_none() {
        return Err("Invalid IPK identity, version or size".into());
    }
    let mut members = ipk.strip_prefix(b"!<arch>\n").ok_or_else(invalid)?;
    if ar_member(&mut members, "debian-binary") != Some(b"2.0\n") {
        return Err(invalid());
    }
    let control_tar = ar_member(&mut members, "control.tar.gz").ok_or_else(invalid)?;
    let data_tar = ar_member(&mut members, "data.tar.gz").ok_or_else(invalid)?;
    if !members.is_empty() || control_tar.len() > MAX_CONTROL as usize {
        return Err(invalid());
    }

    let mut control = String::new();
    archive(control_tar, MAX_CONTROL, |name, directory, _, reader| {
        match (name, directory) {
            ("", true) => {}
            ("control", false) => {
                reader.read_to_string(&mut control).map_err(|_| invalid())?;
            }
            _ => return Err("Unexpected IPK control entry or installation script".into()),
        }
        Ok(())
    })?;
    let fields = control_fields(&control)?;
    if fields.get("package") != Some(&id) || fields.get("version") != Some(&version) {
        return Err("IPK package identity or version does not match the request".into());
    }
    let architecture = match fields.get("architecture").copied() {
        Some("arm") => "arm",
        Some("all") if id != APP => "all",
        _ => return Err("Unsupported IPK architecture".into()),
    };
    verify_payload(data_tar, id, version)?;
    Ok(architecture)
}
