use std::io;

use regex::Regex;

pub(super) fn read_stdin() -> Result<String, String> {
    let mut ret = String::new();
    io::stdin().read_line(&mut ret).map_err(|e| e.to_string())?;
    Ok(ret.trim().to_string())
}

pub(super) fn check_regex(re: &str, text: &str, err_message: &str) -> Result<(), String> {
    Regex::new(re)
        .map_err(|e| e.to_string())?
        .find(text)
        .ok_or(err_message.to_string())
        .map(|_| ())
}