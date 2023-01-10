#[derive(Debug, PartialEq)]
pub enum ValidationError {
    TooShort,
    TooLong,
    ContainsProfanity(Vec<String>),
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let msg = match self {
            Self::TooShort => {
                format!("Username must have a minimum length of {MIN_LENGTH} characters.")
            }
            Self::TooLong => {
                format!("Username cannot exceed a maximum length of {MAX_LENGTH} characters.")
            }
            Self::ContainsProfanity(profanity) => {
                let formatted_profanity = profanity.join(", ");
                format!(
                    "The entered username contains the following disallowed words: [{}]",
                    formatted_profanity
                )
            }
        };

        write!(f, "{}", msg)
    }
}

const MIN_LENGTH: usize = 3;
const MAX_LENGTH: usize = 20;

const PROFANITY: &str = include_str!("profanity.txt");

pub fn validate_username(name: &str) -> Result<(), ValidationError> {
    if name.len() < MIN_LENGTH {
        return Err(ValidationError::TooShort);
    } else if name.len() > MAX_LENGTH {
        return Err(ValidationError::TooLong);
    }

    if let Some(found_profanity) = find_profanity(name) {
        return Err(ValidationError::ContainsProfanity(found_profanity));
    }

    Ok(())
}

fn find_profanity(val: &str) -> Option<Vec<String>> {
    let prof_array: Vec<&str> = PROFANITY.split_whitespace().collect();

    let lowercase_val = val.to_lowercase();
    let found_profanity: Vec<String> = prof_array
        .iter()
        .filter(|profanity| lowercase_val.find(*profanity).is_some())
        .map(|p| p.to_string())
        .collect();

    if found_profanity.is_empty() {
        None
    } else {
        Some(found_profanity)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acceptable_usernames() {
        const EXPECTED: Result<(), ValidationError> = Ok(());
        let names_to_test = [
            "Alaric",
            "Alaric Ganthaeter",
            "Yslith",
            "Tyrlia",
            "CaptainJaeger",
            "Captain Jaeger",
        ];

        names_to_test.iter().for_each(|name| {
            let result = validate_username(name);
            assert_eq!(result, EXPECTED, "{name} should be accepted as valid.");
        });
    }

    #[test]
    fn test_too_short_usernames() {
        const EXPECTED: Result<(), ValidationError> = Err(ValidationError::TooShort);
        let names_to_test = ["Al", "H3", "A", "CJ"];

        names_to_test.iter().for_each(|name| {
            let result = validate_username(name);
            assert_eq!(result, EXPECTED, "{name} should be too short.");
        });
    }

    #[test]
    fn test_too_long_usernames() {
        const EXPECTED: Result<(), ValidationError> = Err(ValidationError::TooLong);
        let names_to_test = [
            "AlaricAlaricAlaricAlaricAlaricAlaric",
            "CaptainJaegerCaptainJaegerCaptainJaegerCaptainJaegerCaptainJaeger",
        ];

        names_to_test.iter().for_each(|name| {
            let result = validate_username(name);
            assert_eq!(result, EXPECTED, "{name} should be too long.");
        });
    }

    #[test]
    fn test_profane_usernames() {
        PROFANITY.split_whitespace().for_each(|profanity| {
            let expected = Err(ValidationError::ContainsProfanity(vec![
                profanity.to_string()
            ]));
            let name = &format!("AB{profanity}cd");
            let result = validate_username(name);
            assert_eq!(result, expected, "{name} should be rejected for profanity.");
        });
    }

    #[test]
    fn test_capitalized_profane_usernames() {
        PROFANITY
            .split_whitespace()
            .map(|s| s.to_uppercase())
            .for_each(|profanity| {
                let expected = Err(ValidationError::ContainsProfanity(vec![profanity
                    .to_lowercase()
                    .to_string()]));
                let name = &format!("AB{profanity}cd");
                let result = validate_username(name);
                assert_eq!(result, expected, "{name} should be rejected for profanity.");
            });
    }
}
