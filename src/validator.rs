use std::str::FromStr;

use email_address::EmailAddress;
use inquire::validator::{ErrorMessage, StringValidator, Validation};

#[derive(Clone)]
pub struct EmailValidator;

impl StringValidator for EmailValidator {
    fn validate(&self, input: &str) -> Result<Validation, inquire::CustomUserError> {
        if let Err(e) = EmailAddress::from_str(input) {
            Ok(Validation::Invalid(ErrorMessage::Custom(format!("{e}"))))
        } else {
            Ok(Validation::Valid)
        }
    }
}
