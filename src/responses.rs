pub enum Responses {
    JWTTokenError,
    JWTTokenCreationError,
    NoAuthHeaderError,
    InvalidAuthHeaderError,
    NoPermissionError,
    UserAlreadyExistsError,
    UserDoesNotExistOrWrongCredentialsError,
    LoginSuccessfull,
}

impl Responses {
    pub fn as_str(&self) -> &'static str {
        match self {
            Responses::JWTTokenError => "JWT token error",
            Responses::JWTTokenCreationError => "JWT token creation error",
            Responses::NoAuthHeaderError => "No authorization header",
            Responses::InvalidAuthHeaderError => "Invalid authorization header",
            Responses::NoPermissionError => "No permission",
            Responses::UserDoesNotExistOrWrongCredentialsError => {
                "User does not exist or wrong password"
            }
            Responses::LoginSuccessfull => "Login successfull!",
            Responses::UserAlreadyExistsError => todo!(),
        }
    }
}
