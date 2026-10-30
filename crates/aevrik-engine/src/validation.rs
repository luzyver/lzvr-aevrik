use crate::{
    error::{Result, invalid},
    model::{ProjectInput, ServiceInput},
};
use std::path::{Path, PathBuf};

pub(crate) fn directory(value: &str) -> Result<PathBuf> {
    let path = Path::new(value.trim());
    if !path.is_absolute() || !path.is_dir() {
        return Err(invalid("Choose an existing absolute directory."));
    }
    Ok(path.canonicalize()?)
}
fn name(value: &str) -> Result<String> {
    let text = value.trim();
    if text.is_empty() || text.chars().count() > 120 || text.chars().any(char::is_control) {
        return Err(invalid(
            "Use a name between 1 and 120 characters without control characters.",
        ));
    }
    Ok(text.into())
}
pub(crate) fn project(input: &mut ProjectInput) -> Result<()> {
    input.name = name(&input.name)?;
    input.path = directory(&input.path)?.to_string_lossy().into();
    input.domain = input.domain.trim().to_ascii_lowercase();
    let valid = input.domain.ends_with(".test")
        && input.domain.len() <= 253
        && input.domain.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b == b'-')
        });
    if !valid {
        return Err(invalid("Use a valid .test domain, for example atlas.test."));
    }
    if input.runtime.len() > 80 || input.runtime.chars().any(char::is_control) {
        return Err(invalid("Runtime label is invalid."));
    }
    Ok(())
}
pub(crate) fn service(input: &mut ServiceInput) -> Result<()> {
    input.name = name(&input.name)?;
    let path = Path::new(input.executable.trim());
    if !path.is_absolute() || !path.is_file() {
        return Err(invalid(
            "Choose an existing executable using its full path.",
        ));
    }
    #[cfg(windows)]
    if !path
        .extension()
        .is_some_and(|x| x.eq_ignore_ascii_case("exe"))
    {
        return Err(invalid(
            "Select an .exe file. Shell scripts and command strings are not supported.",
        ));
    }
    input.executable = path.canonicalize()?.to_string_lossy().into();
    input.cwd = directory(&input.cwd)?.to_string_lossy().into();
    if input.port == Some(0) {
        return Err(invalid("Port must be between 1 and 65535."));
    }
    if input.args.len() > 128
        || input.args.iter().map(String::len).sum::<usize>() > 24_000
        || input.args.iter().any(|x| x.contains('\0'))
    {
        return Err(invalid(
            "Arguments exceed the supported size or contain a null character.",
        ));
    }
    Ok(())
}
