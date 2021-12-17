use std::fs::OpenOptions;
use std::io::Read;

use super::utils::{check_regex, read_stdin};

#[allow(non_camel_case_types)]
type MD5_String = String;
type CredentialsMap = std::collections::BTreeMap<MD5_String, (MD5_String, MD5_String)>;

pub(super) struct Registrar;

impl Registrar {
    const LOGIN_MESSAGE: &'static str = "Введите логин:";
    const PASSWORD_MESSAGE: &'static str = "Введите пароль:";
    const ACCESS_LEVEL_MESSAGE: &'static str = "Введите Уровень доступа:";
    const CREDENTIALS_PATH: &'static str = "credentials.txt";

    pub(super) fn register() -> Result<(), String> {
        loop {
            let login = Self::read_registration_data(
                Self::LOGIN_MESSAGE,
                "Failed reading user input login",
            )?;
            let password = Self::read_registration_data(
                Self::PASSWORD_MESSAGE,
                "Failed reading user input password",
            )?;
            {
                // Check password
                if login == password {
                    return Err("Login is equal to password".to_string());
                }
                Self::check_password(&password)?
            };
            let access_level = {
                let raw = Self::read_registration_data(
                    Self::ACCESS_LEVEL_MESSAGE,
                    "Failed reading user access level",
                )?;
                Self::convert_to_access_lvl(&raw)?
            };

            // TODO
            // 1. Create a different abstraction (for example, CredentialsManager),
            // which manages (reads/writes) credentials in a separate module.
            // 2. Make sure there are no copy paste and there are no abstractions leak
            let mut credentials = {
                let mut contents = String::new();
                let mut f = OpenOptions::new()
                    .read(true)
                    .open(Self::CREDENTIALS_PATH)
                    .map_err(|e| format!("Failed opening file {}: {}", Self::CREDENTIALS_PATH, e))?;
                f.read_to_string(&mut contents)
                    .map_err(|e| format!("Failed to read from file: {:?}", e))?;

                if contents.len() == 0 {
                    CredentialsMap::new()
                } else {
                    serde_json::de::from_str(&contents)
                        .map_err(|e| format!("Failed deserializing credentials data: {:?}", e))?
                }
            };
            if credentials
                .insert(super::md5_utf8(&login), (super::md5_utf8(&password), access_level))
                .is_some()
            {
                // If user exists, start again
                println!("Such user exists. Performing registration again");
            } else {
                let f = OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .open(Self::CREDENTIALS_PATH)
                    .map_err(|e| format!("Failed opening file {}: {}", Self::CREDENTIALS_PATH, e))?;
                serde_json::ser::to_writer(f, &credentials)
                    .map_err(|e| format!("Failed serializing credentials data: {:?}", e))?;
                break;
            }
        }
        Ok(())
    }

    fn read_registration_data(prompt_msg: &str, app_err: &str) -> Result<String, String> {
        println!("{}\t", prompt_msg);
        read_stdin().map_err(|native_err| format!("{}: {}", app_err, native_err))
    }

    fn check_password(password: &str) -> Result<(), String> {
        Self::check_length(password)?;
        Self::check_symbols(password)
    }

    fn check_length(password: &str) -> Result<(), String> {
        if password.len() <= 7 {
            return Err("Password length is less than 8 symbols".to_string());
        }
        Ok(())
    }

    fn check_symbols(password: &str) -> Result<(), String> {
        // Check upper case letters
        check_regex(
            r"[A-Z]",
            password,
            "Check password has upper case letters failed",
        )?;
        // Check upper case letters
        check_regex(
            r"[a-z]",
            password,
            "Check password has lower case letters failed",
        )?;
        // Check numbers
        check_regex(r"[0-9]", password, "Check password has numbers failed")?;
        // Check has special symbols
        let re = format!("[{}]", regex::escape("!№@#$%^&*():;[]?*()-_=+{},.\""));
        check_regex(&re, password, "Check password has special symbols failed")?;
        // Check has no spaces
        check_regex(r"^\S*$", password, "Check password has no spaces failed")
    }

    fn convert_to_access_lvl(lvl: &str) -> Result<String, String> {
        match lvl {
            "1" => Ok("admin".to_string()),
            "2" => Ok("user".to_string()),
            _ => Err("There is no such access level".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Registrar;

    // TODO
    // 1. Write tests, to check all cases in "check_symbols"
    // 2. Write test for checking passwords using each of special_symbol
    // "@", r#"""#, "#", "№", r#"$"#, ";", "%", "^", ":", ";", "&", "?", "*", "(", ")", "_",
    // "-", "=", "+", "{", "}", ".", ",", "!",

    #[test]
    fn check_password_length() {
        let password = "some_pa\n";
        assert_eq!(password.len(), 8);
        assert!(Registrar::check_length(password).is_ok());
        assert!(Registrar::check_length(password.trim()).is_err())
    }

    #[test]
    fn check_password_is_valid() {
        let password_valid = "somePa@_ss1";
        let invalid_passwords = [
            "somepass",
            "SOMEPASS",
            "Somepass",
            "somePass1",
            "somePa@ ss1",
        ];

        assert!(Registrar::check_symbols(password_valid).is_ok());

        for invalid_pass in invalid_passwords {
            assert!(Registrar::check_symbols(invalid_pass).is_err());
        }
    }

}