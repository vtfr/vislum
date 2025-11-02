#[macro_export]
macro_rules! impl_extensions {
    (
        $(#[$meta:meta])*
        $vis:vis struct $ident:ident {
            $(
                $field:ident => $ext_c_str:expr,
            )*
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd, Default)]
        $vis struct $ident {
            $(
                $field: bool,
            )*
        }

        impl $ident {
            #[doc = concat!("Creates a new empty ", stringify!($ident), ".")]
            pub const fn empty() -> Self {
                Self {
                    $(
                        $field: false,
                    )*
                }
            }

            #[doc = concat!("Creates a new ", stringify!($ident), " from an iterator of extension names.")]
            pub fn from_iter<'a>(extensions: impl Iterator<Item = &'a std::ffi::CStr>) -> Self {
                let mut result = Self::empty();
                for extension in extensions {
                    $(
                        if extension == $ext_c_str {
                            result.$field = true;
                            continue;
                        }
                    )*
                }

                result
            }

            // Returns an iterator over the enabled extensions.
            pub fn iter_c_strs(&self) -> impl Iterator<Item = &'static std::ffi::CStr> {
                [
                    $(
                        ($ext_c_str, self.$field),
                    )*
                ]
                    .into_iter()
                    .filter_map(|(ext_c_str, enabled)| enabled.then_some(ext_c_str))
            }

            // Returns an iterator over the pointers to the enabled extensions.
            //
            // # Safety
            // All pointers returned in this iterator are valid for the lifetime
            // of the program, as these were obtained from the static [`CStr`]s
            // defined in the ash crate.
            //
            // [`CStr`]: std::ffi::CStr
            pub fn iter_c_ptrs(&self) -> impl Iterator<Item = *const i8> {
                self.iter_c_strs().map(|e| e.as_ptr())
            }

            /// Returns an iterator over the enabled extensions as strings.
            pub fn iter_strs(&self) -> impl Iterator<Item = &'static str> {
                self.iter_c_strs().map(|e| e.to_str().unwrap())
            }

            #[doc = concat!("Returns `true` if this is empty.")]
            pub const fn is_empty(&self) -> bool {
                true $( && !self.$field )*
            }

            #[doc = concat!("Computes the difference between this and another ", stringify!($ident), ".")]
            pub const fn difference(&self, other: &Self) -> Self {
                Self {
                    $(
                        $field: self.$field && !other.$field,
                    )*
                }
            }

            #[doc = concat!("Computes the intersection of this and another ", stringify!($ident), ".")]
            pub const fn intersection(&self, other: &Self) -> Self {
                Self {
                    $(
                        $field: self.$field && other.$field,
                    )*
                }
            }
        }

        impl std::fmt::Display for $ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut first = true;

                write!(f, "[")?;
                for name in self.iter_strs() {
                    if first {
                        write!(f, "{}", name)?;
                        first = false;
                    } else {
                        write!(f, ", {}", name)?;
                    }
                }

                write!(f, "]")
            }
        }
    };
}
