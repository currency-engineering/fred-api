use std::error::Error as StdError;
use std::fmt;
use std::mem;

// `ErrorKind` uses only owned strings or copy types as parameters. This means
// that `Error`s in client code do not require a lifetime, which could cause
// trouble if the error is wrapped and passed around.
#[derive(Clone, Debug)]
pub struct Error(pub Box<ErrorKind>);

impl Error {
    pub fn new(kind: ErrorKind) -> Error {
        Error(Box::new(kind))
    }

    pub fn kind(&self) -> ErrorKind {
        *self.0.clone()
    }

    // // Maps InnerBadIndent to BadIndent
    // pub fn from_inner<T: Display>(
    //     self,
    //     token: T,
    //     line: usize) -> Error
    // {
    //     match self.kind() {
    //         ErrorKind::InnerBadIndent(indent) => {
    //             Error::new(ErrorKind::BadIndent(token.to_string(), line, indent))
    //         },
    //         _ => self,
    //     }
    // }
}

#[derive(Clone, Debug)]
pub enum ErrorKind {
    Reqwest(String),
    FredResponse(String),
}

// Equal if they are the same kind.
impl PartialEq for ErrorKind {
    fn eq(&self, other: &Self) -> bool {
        mem::discriminant(self) == mem::discriminant(other)
    }  
}

impl StdError for Error {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match *self.0 {

            ErrorKind::Reqwest(_) => None,
            ErrorKind::FredResponse(_) => None,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &*self.0 {

            ErrorKind::Reqwest(msg) => {
                write!(
                    f, 
                    "Reqwest error: {}",
                    msg,
                )
            },

            ErrorKind::FredResponse(msg) => {
                write!(
                    f, 
                    "Json error: {}",
                    msg,
                )
            }
        }
    }
}

