macro_rules! enum_error {
    ($enum_error:ident { $( $variant:ident($error:path) ),+ $( , )? }) => {
        use std::{fmt::{self, Display, Formatter}};

        #[derive(Debug)]
        pub(crate) enum $enum_error {
            $( $variant($error) ),+
        }

        impl Display for $enum_error {
            fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
                match self {
                    err => err.fmt(f),
                }
            }
        }

        impl std::error::Error for $enum_error {
            fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
                match self {
                    err => err.source(),
                }
            }
        }

        $(
            impl From<$error> for $enum_error {
                fn from(value: $error) -> Self {
                    Self::$variant(value)
                }
            }
        )+
    };
}
