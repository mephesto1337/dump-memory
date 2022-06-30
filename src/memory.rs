use std::fmt;
use std::path::Path;
use std::str::FromStr;

use crate::{Error, Result};

/// Standard permissions for a region
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
#[repr(u32)]
pub enum PermissionBits {
    Read = 1,
    Write = 2,
    Exec = 4,
    Private = 8,
    Shared = 16,
}

impl fmt::Display for PermissionBits {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Read => f.write_str("r"),
            Self::Write => f.write_str("w"),
            Self::Exec => f.write_str("e"),
            Self::Private => f.write_str("p"),
            Self::Shared => f.write_str("s"),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Permissions(u32);

impl Permissions {
    pub fn has_perm(&self, pbit: PermissionBits) -> bool {
        self.0 & pbit as u32 != 0
    }

    pub fn new() -> Self {
        Self(0)
    }

    pub fn add(&mut self, pbit: PermissionBits) -> &mut Self {
        self.0 = self.0 | pbit as u32;
        self
    }
}

impl fmt::Display for Permissions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for pbit in &[
            PermissionBits::Read,
            PermissionBits::Write,
            PermissionBits::Exec,
            PermissionBits::Private,
            PermissionBits::Shared,
        ][..]
        {
            if self.has_perm(*pbit) {
                write!(f, "{}", pbit)?;
            } else {
                f.write_str("-")?;
            }
        }

        Ok(())
    }
}

macro_rules! match_char {
    ($field:expr, $subfield:expr, $n:expr, $permissions:ident, $($c:expr => $v:expr),*) => {
        match $n {
            $(Some($c) => $v,)*
            Some(c) => {
                return Err(Error::MalformedRegionField {
                    field: concat!($field, "-", $subfield),
                    value: c.into(),
                })
            }
            None => {
                return Err(Error::MalformedRegionField {
                    field: $field,
                    value: $permissions.into(),
                })
            }
        }
    };
}

impl FromStr for Permissions {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut chars = s.chars();
        let mut perms = Permissions::new();

        match_char!("permissions", "read", chars.next(), s, 'r' => {perms.add(PermissionBits::Read);}, '-' => {});
        match_char!("permissions", "write", chars.next(), s, 'w' => {perms.add(PermissionBits::Write);}, '-' => {});
        match_char!("permissions", "exec", chars.next(), s, 'x' => {perms.add(PermissionBits::Exec);}, '-' => {});
        match_char!("permissions", "private/shared", chars.next(), s, 'p' => {perms.add(PermissionBits::Private);}, 's' => {perms.add(PermissionBits::Shared);});

        Ok(perms)
    }
}

/// Represents a Linux device
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Device {
    pub major: u8,
    pub minor: u8,
}

/// A memory region
#[derive(Debug, PartialEq, Eq)]
pub struct Region {
    /// Start address
    pub start: usize,

    /// End address
    pub end: usize,

    /// bit-or combinaison of `Permissions`
    pub perms: Permissions,

    /// Offset in file
    pub offset: usize,

    /// Device associated if any
    pub dev: Device,

    /// Inode on the device
    pub inode: u64,

    /// Backing file if any
    pub file: Option<String>,
}

impl FromStr for Region {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut parts = s.split_ascii_whitespace();

        let start_end = parts.next().ok_or(Error::MissingRegionField("start-end"))?;
        let permissions = parts
            .next()
            .ok_or(Error::MissingRegionField("permissions"))?;
        let offset = parts.next().ok_or(Error::MissingRegionField("offset"))?;
        let device = parts.next().ok_or(Error::MissingRegionField("device"))?;
        let inode = parts.next().ok_or(Error::MissingRegionField("inode"))?;
        let file = parts.next().map(|f| f.to_owned());

        let (start, end) =
            start_end
                .split_once('-')
                .ok_or_else(|| Error::MalformedRegionField {
                    field: "start-end",
                    value: start_end.into(),
                })?;
        let start = usize::from_str_radix(start, 16)?;
        let end = usize::from_str_radix(end, 16)?;

        let perms = permissions.parse()?;

        let offset = usize::from_str_radix(offset, 16)?;

        let (major, minor) =
            device
                .rsplit_once(':')
                .ok_or_else(|| Error::MalformedRegionField {
                    field: "device",
                    value: device.into(),
                })?;
        let dev = Device {
            major: u8::from_str_radix(major, 16)?,
            minor: u8::from_str_radix(minor, 16)?,
        };

        let inode = inode.parse()?;

        Ok(Self {
            start,
            end,
            perms,
            offset,
            dev,
            inode,
            file,
        })
    }
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(filename) = self.filename() {
            write!(
                f,
                "{:x}-{:x}_{}_+{:x}_{:02x}:{:02x}_{:x}_{}",
                self.start,
                self.end,
                self.perms,
                self.offset,
                self.dev.major,
                self.dev.minor,
                self.inode,
                filename
            )
        } else {
            write!(
                f,
                "{:x}-{:x}_{}_+{:x}_{:02x}:{:02x}_{:x}",
                self.start,
                self.end,
                self.perms,
                self.offset,
                self.dev.major,
                self.dev.minor,
                self.inode,
            )
        }
    }
}

impl Region {
    pub fn size(&self) -> usize {
        self.end - self.start
    }

    pub fn filename(&self) -> Option<&str> {
        self.file()
            .map(|f| Path::new(f))
            .and_then(|p| p.file_name())
            .and_then(|os| os.to_str())
    }
    pub fn file(&self) -> Option<&str> {
        self.file.as_ref().map(|s| s.as_str())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Memory {
    pid: u32,
    regions: Vec<Region>,
}

impl std::ops::Deref for Memory {
    type Target = [Region];

    fn deref(&self) -> &Self::Target {
        &self.regions[..]
    }
}

impl Memory {
    pub fn from_pid(pid: u32) -> Result<Self> {
        let maps = std::fs::read_to_string(format!("/proc/{}/maps", pid))?;
        let mut regions = Vec::new();

        for line in maps.lines() {
            regions.push(line.parse::<Region>()?);
        }
        Ok(Self { pid, regions })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_region_with_file() {
        let region_with_file = "559213685000-5592136ff000 r--p 00000000 fe:01 1462190                    /usr/bin/nvim";
        let region = Region {
            start: 0x559213685000,
            end: 0x5592136ff000,
            perms: Permissions(PermissionBits::Read as u32 | PermissionBits::Private as u32),
            offset: 0,
            dev: Device {
                major: 0xfe,
                minor: 0x01,
            },
            inode: 1462190,
            file: Some("/usr/bin/nvim".into()),
        };
        let parsed_region = match region_with_file.parse::<Region>() {
            Ok(region) => region,
            Err(e) => {
                panic!("{}", e);
            }
        };
        assert_eq!(region, parsed_region);
    }

    #[test]
    fn test_parse_region_without_file() {
        let region_with_file = "559213ad7000-559213af0000 rw-s 0000d000 00:00 0 ";
        let region = Region {
            start: 0x559213ad7000,
            end: 0x559213af0000,
            perms: Permissions(
                PermissionBits::Read as u32
                    | PermissionBits::Write as u32
                    | PermissionBits::Shared as u32,
            ),
            offset: 0xd000,
            dev: Device {
                major: 0x0,
                minor: 0x0,
            },
            inode: 0,
            file: None,
        };
        let parsed_region = match region_with_file.parse::<Region>() {
            Ok(region) => region,
            Err(e) => {
                panic!("{}", e);
            }
        };
        assert_eq!(region, parsed_region);
    }
}
