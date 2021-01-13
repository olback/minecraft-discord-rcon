#[macro_export]
macro_rules! new_err {
    ($e:expr) => {
        $crate::error::Error::new(format!("{}", $e), std::file!(), std::line!())
    };
}

#[macro_export]
macro_rules! impl_from {
    ($t:ty) => {
        impl From<$t> for Error {
            fn from(err: $t) -> Error {
                super::new_err!(format!("{}", err))
            }
        }
    };
}

#[derive(Debug)]
pub struct Error {
    cause: String,
    file: String,
    line: u32,
}

impl Error {
    pub fn new<C: Into<String>>(cause: C, file: &str, line: u32) -> Self {
        Self {
            cause: cause.into(),
            file: String::from(file),
            line,
        }
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}#{}: {}", self.file, self.line, self.cause)
    }
}

impl_from!(rcon::Error);
impl_from!(std::io::Error);
impl_from!(discord::Error);
