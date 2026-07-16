pub trait MapErrorToString<T> {
    fn map_err_to_string(self) -> Result<T, String>;
}

impl<T, E: ToString> MapErrorToString<T> for Result<T, E> {
    fn map_err_to_string(self) -> Result<T, String> {
        self.map_err(|error| error.to_string())
    }
}
