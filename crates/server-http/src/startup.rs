//! Bounded startup file reads over an operator-owned immutable deployment tree.

use crate::external;
use stackpress_lib::{Error, Result};
use std::{
    fs,
    io::Read,
    path::{Component, Path},
};

/// Refuse symlink components and special files before opening; reads still enforce their byte cap.
/// Portable metadata/open is not atomic against hostile concurrent tree mutation.
pub(crate) fn read_regular(path: &Path, max: usize) -> Result<Vec<u8>> {
    if max == 0 || max == usize::MAX {
        return Err(Error::invalid("startup file cap invalid"));
    }
    let mut prefix = std::path::PathBuf::new();
    for component in path.components() {
        if matches!(component, Component::ParentDir) {
            return Err(Error::invalid("startup parent path unsupported"));
        }
        prefix.push(component);
        let metadata = fs::symlink_metadata(&prefix)
            .map_err(|cause| external("startup path metadata failed", cause))?;
        if metadata.file_type().is_symlink() {
            return Err(Error::invalid("startup symlink unsupported"));
        }
    }
    let metadata = fs::symlink_metadata(path)
        .map_err(|cause| external("startup file metadata failed", cause))?;
    if !metadata.is_file() {
        return Err(Error::invalid("startup input must be a regular file"));
    }
    if metadata.len() > max as u64 {
        return Err(Error::limit("startup file byte limit exceeded"));
    }
    let file = fs::File::open(path).map_err(|cause| external("startup file open failed", cause))?;
    if !file
        .metadata()
        .map_err(|cause| external("opened startup metadata failed", cause))?
        .is_file()
    {
        return Err(Error::invalid("opened startup input must be regular"));
    }
    let mut bytes = Vec::new();
    file.take(max as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|cause| external("startup file read failed", cause))?;
    if bytes.len() > max {
        return Err(Error::limit("startup file grew beyond limit"));
    }
    Ok(bytes)
}
