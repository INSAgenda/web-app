 /// Check if the user gave by the user is considered safe
 pub fn is_safe_password(password: &str) -> Result<(), Vec<InvalidPasswordError>> {
    let mut errors = Vec::new();

    // Check length
    if password.len() < 10 || password.len() > 128 {
        errors.push(InvalidPasswordError::PasswordLength);
    }
    // Check if contain a lowercase and a uppercase letter
    if password == password.to_lowercase() || password == password.to_uppercase() {
        errors.push(InvalidPasswordError::NoUppercaseAndLowercase);
    }
    // Check if contain a number 
    if !password.chars().any(|c| c.is_ascii_digit()) {
        errors.push(InvalidPasswordError::NoDigit);
    }
    // Check if contain a special character
    if !password.chars().any(|c| c.is_ascii_punctuation() || c == ' ') {
        errors.push(InvalidPasswordError::NoSpecialChar);
    }

    if errors.len() > 1 || (!errors.is_empty() && errors[0] == InvalidPasswordError::PasswordLength) { 
        Err(errors)
    }else{
        Ok(())
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum InvalidPasswordError {
    PasswordLength,
    TopPassword,
    NoDigit,
    NoSpecialChar,
    NoUppercaseAndLowercase,
}

impl InvalidPasswordError {
    // Get the error message for the error (en_message, fr_message)
    pub fn to_error_message(&self) -> (String, String) {
        match self {
            InvalidPasswordError::PasswordLength => (
                "The password must be between 10 and 128 characters long".to_string(),
                "Le mot de passe doit posséder entre 10 et 128 caractères".to_string(),
            ),
            InvalidPasswordError::NoUppercaseAndLowercase => (
                "The password must contain at least one uppercase and one lowercase letter".to_string(),
                "Le mot de passe doit contenir au moins une lettre majuscule et une lettre minuscule".to_string(),
            ),
            InvalidPasswordError::NoDigit => (
                "The password must contain at least one digit".to_string(),
                "Le mot de passe doit contenir au moins un chiffre".to_string(),
            ),
            InvalidPasswordError::NoSpecialChar => (
                "The password must contain at least one special character".to_string(),
                "Le mot de passe doit contenir au moins un caractère spécial".to_string(),
            ),
            InvalidPasswordError::TopPassword => (
                "The password is too common".to_string(),
                "Le mot de passe est trop commun".to_string(),
            ),
        }
    }
}

pub fn compact_errors(err: &Vec<InvalidPasswordError>) -> (String, String){
    let mut en_message = String::new();
    let mut fr_message = String::new();
    for e in err {
        let (en, fr) = e.to_error_message();
        en_message.push_str(&en);
        en_message.push_str("<br>");
        fr_message.push_str(&fr);
        fr_message.push_str("<br>");
    }
    (en_message, fr_message)
}

#[cfg(test)]
mod tests{
    #[test]
    fn test_is_safe_password() {
        use super::*;
        let res = is_safe_password("aaaaa");
        if let Err(err) = res {
            assert!(err.iter().any(|e| matches!(e, InvalidPasswordError::PasswordLength)));
            assert!(err.iter().any(|e| matches!(e, InvalidPasswordError::NoDigit)));
            assert!(err.iter().any(|e| matches!(e, InvalidPasswordError::NoSpecialChar)));
            assert!(err.iter().any(|e| matches!(e, InvalidPasswordError::NoUppercaseAndLowercase)));
        } else {
            panic!("Password should be too short");
        }

        let res = is_safe_password("Password12345!"); // Password to ban (top password)
        assert!(res.is_ok());
        
    }
}