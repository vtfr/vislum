#[macro_export]
macro_rules! impl_extensions {
    (
        $(#[$attr:meta])*
        $vis:vis struct $ident:ident {
            $(
                $(#[$field_meta:meta])* 
                $field_vis:vis $field_ident:ident = $ext_path:path
            ),*
            $(,)?
        }
    ) => {
        $(#[$attr])*
        #[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
        $vis struct $ident {
            $(
                $(#[$field_meta])*
                $field_vis $field_ident: bool,
            )*
        }

        impl $ident {
            /// Create the extensions from an iterator of extension names.
            pub fn from_vk<'a>(iter: impl Iterator<Item = &'a std::ffi::CStr>) -> Self {
                let mut extensions = Self::default();
                for ext in iter {
                    $(
                        if ext == $ext_path {
                            extensions.$field_ident = true;
                            continue;
                        }
                    )*
                }
                extensions
            }

            /// Convert the extensions to an iterator of extension names.
            pub fn to_vk(&self) -> impl Iterator<Item = &'static std::ffi::CStr> {
                [
                    $(
                        ($ext_path, self.$field_ident)
                    ),*
                ]
                    .into_iter()
                    .filter(|(_, enabled)| *enabled)
                    .map(|(ext, _)| ext)
            }

            pub fn combine(&self, other: &Self) -> Self {
                Self {
                    $(
                        $field_ident: self.$field_ident || other.$field_ident,
                    )*
                }
            }

            pub fn difference(&self, other: &Self) -> Self {
                Self {
                    $(
                        $field_ident: self.$field_ident && !other.$field_ident,
                    )*
                }
            }

            pub fn is_empty(&self) -> bool {
                self == &Self::default()
            }
        }
    }
}

#[macro_export]
macro_rules! vk_enum {
    (
        $(#[$attr:meta])*
        $vis:vis enum $ident:ident: $vk_type:ty {
            $(
                $(#[$field_meta:meta])*
                $variant_ident:ident = $vk_ident:ident
            ),*
            $(,)?
        }
    ) => {
        $(#[$attr])*
        #[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
        $vis enum $ident {
            $(
                $(#[$field_meta])*
                $variant_ident,
            )*
        }

        impl std::fmt::Display for $ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(match self {
                    $(
                        <Self>::$variant_ident => stringify!($variant_ident),
                    )*
                })
            }
        }

        impl $ident {
            pub const fn from_vk(value: $vk_type) -> Option<Self> {
                match value {
                    $(
                        <$vk_type>::$vk_ident => Some(<Self>::$variant_ident),
                    )*
                    _ => None,
                }
            }

            pub const fn to_vk(self) -> $vk_type {
                match self {
                    $(
                        <Self>::$variant_ident => <$vk_type>::$vk_ident,
                    )*
                }
            }
        }
    }
}

#[macro_export]
macro_rules! impl_features {
    (
        $(#[$attr:meta])*
        $vis:vis struct $ident:ident {
            $(
                $(#[$field_meta:meta])*
                pub $field_ident:ident: bool
            ),*
            $(,)?
        }
    ) => {
        $(#[$attr])*
        #[derive(Debug, Default, Copy, Clone, PartialEq, Eq, Hash)]
        $vis struct $ident {
            $(
                $(#[$field_meta])*
                pub $field_ident: bool,
            )*
        }

        impl $ident {
            pub fn combine(&self, other: &Self) -> Self {
                Self {
                    $(
                        $field_ident: self.$field_ident || other.$field_ident,
                    )*
                }
            }

            pub fn difference(&self, other: &Self) -> Self {
                Self {
                    $(
                        $field_ident: self.$field_ident && !other.$field_ident,
                    )*
                }
            }

            pub fn is_empty(&self) -> bool {
                self == &Self::default()
            }
        }
    }
}
