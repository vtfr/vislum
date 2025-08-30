#[macro_export]
macro_rules! new_uuid_type {
    {
        $(#[$meta:meta])*
        $pub:vis struct $name:ident;
    } => {
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
        $(#[$meta])*
        $pub struct $name(uuid::Uuid);

        impl $name {
            /// Creates a new random id.
            pub fn new() -> Self {
                Self(uuid::Uuid::new_v4())
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }
    };
}