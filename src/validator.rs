use std::str::FromStr;

use email_address::EmailAddress;
use inquire::{
    list_option::ListOption,
    validator::{ErrorMessage, MultiOptionValidator, StringValidator, Validation},
    CustomUserError,
};

#[derive(Clone)]
pub struct EmailValidator;

impl StringValidator for EmailValidator {
    fn validate(&self, input: &str) -> Result<Validation, CustomUserError> {
        if let Err(e) = EmailAddress::from_str(input) {
            Ok(Validation::Invalid(ErrorMessage::Custom(format!("{e}"))))
        } else {
            Ok(Validation::Valid)
        }
    }
}

#[derive(Clone)]
pub struct NonEmptyValidator;

impl<T> MultiOptionValidator<T> for NonEmptyValidator {
    fn validate(&self, input: &[ListOption<&T>]) -> Result<Validation, CustomUserError> {
        if input.is_empty() {
            Ok(Validation::Invalid(ErrorMessage::Custom(
                "Selection can't be empty!".to_string(),
            )))
        } else {
            Ok(Validation::Valid)
        }
    }
}
