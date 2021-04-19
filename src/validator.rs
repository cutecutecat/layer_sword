use std::collections::HashSet;
use std::iter::FromIterator;

/// validator of int argument for clap parser
/// # Examples
///
/// ```rust
/// use layer_sword::validator::valid_int;
///
/// let x = String::from("12");
/// assert_eq!(valid_int(x), Ok(()));
///
/// let x = String::from("128");
/// assert_eq!(valid_int(x), Err("argument 128 is not between -1 to 127".to_string()));
///
/// let x = String::from("me");
/// assert_eq!(valid_int(x), Err("argument is not INT type".to_string()));
/// ```
pub fn valid_int(arg: String) -> Result<(), String> {
    let convert_int = arg.parse::<i16>();
    match convert_int {
        Err(_) => Err("argument is not INT type".to_string()),
        Ok(v) => {
            if v < -1 || v > 127 {
                Err(format!("argument {:} is not between -1 to 127", v))
            } else {
                Ok(())
            }
        }
    }
}

/// validator of string argument for clap parser
/// # Examples
///
/// ```rust
/// use layer_sword::validator::valid_alphabet;
///
/// let x = String::from("12a");
/// assert_eq!(valid_alphabet(x), Ok(()));
///
/// let x = String::from("💝");
/// assert_eq!(valid_alphabet(x), Err("argument has char not in ascii type".to_string()));
///
/// ```
pub fn valid_alphabet(arg: String) -> Result<(), String> {
    let forbidden_char = vec!['/', '\\', '<', '>', ':', '"', '|', '.'];
    let forbidden_char:HashSet<&char> = HashSet::from_iter(forbidden_char.iter());
    if arg.chars().all(|c| { c.is_alphanumeric() && !forbidden_char.contains(&c) }) {
        Ok(())
    } else {
        Err("argument has char not in ascii type".to_string())
    }
}