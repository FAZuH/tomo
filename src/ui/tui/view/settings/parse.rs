use std::path::PathBuf;
use std::time::Duration;

use crate::config::Volume;
use crate::ui::prelude::*;

pub(crate) fn parse_path(s: impl AsRef<str>, cmds: &mut Vec<SettingsCmd>) -> Option<PathBuf> {
    let s = s.as_ref();
    if s.is_empty() {
        return None;
    }
    let path = PathBuf::from(s);
    if !path.exists() {
        cmds.push(SettingsCmd::ShowToast {
            message: "Path does not exist".to_string(),
            r#type: ToastType::Warning,
        });
    }
    Some(path)
}

pub(crate) fn parse_dur(s: impl AsRef<str>) -> Result<Duration, SettingsCmd> {
    try_parse(s, |s| s.parse::<u64>(), "integer").map(|val| Duration::from_secs(val * 60))
}

pub(crate) fn parse_vol(s: impl AsRef<str>) -> Result<Volume, SettingsCmd> {
    let s = s.as_ref();
    if s.is_empty() {
        Ok(Volume::default())
    } else {
        try_parse(s, |s| Volume::try_from(s), "percent")
    }
}

pub(crate) fn try_parse<T, E: std::fmt::Debug>(
    s: impl AsRef<str>,
    f: impl for<'a> FnOnce(&'a str) -> Result<T, E>,
    label: &str,
) -> Result<T, SettingsCmd> {
    let s = s.as_ref();
    f(s).map_err(|e| SettingsCmd::ShowToast {
        message: format!("Failed converting '{s}' to {label}: {e:?}"),
        r#type: ToastType::Error,
    })
}
