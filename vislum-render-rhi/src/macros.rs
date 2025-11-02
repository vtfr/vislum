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
                pub $field: bool,
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

/// Macro to create an enum that maps to/from a Vulkan enum.
#[macro_export]
macro_rules! vk_enum {
    (
        $(#[$meta:meta])*
        $vis:vis enum $ident:ident: $vk_type:ty {
            $(
                $(#[$variant_meta:meta])*
                $variant:ident => $vk_value:ident,
            )*
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
        $vis enum $ident {
            $(
                $(#[$variant_meta])*
                $variant,
            )*
        }

        impl $ident {
            /// Converts from a Vulkan enum value.
            pub const fn from_vk(value: $vk_type) -> Option<Self> {
                match value {
                    $(
                        <$vk_type>::$vk_value => Some(Self::$variant),
                    )*
                    _ => None,
                }
            }

            /// Converts to a Vulkan enum value.
            pub const fn to_vk(self) -> $vk_type {
                match self {
                    $(
                        Self::$variant => <$vk_type>::$vk_value,
                    )*
                }
            }
        }
    };
}

/// Macro to create a flags type that maps to/from Vulkan flags.
#[macro_export]
macro_rules! vk_enum_flags {
    (
        $(#[$meta:meta])*
        $vis:vis struct $ident:ident: $vk_type:ty {
            $(
                $(#[$field_meta:meta])*
                $field:ident => $vk_flag:ident,
            )*
        }
    ) => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
        #[repr(transparent)]
        $vis struct $ident($vk_type);

        impl $ident {
            $(
                $(#[$field_meta])*
                pub const $field: Self = Self(<$vk_type>::$vk_flag);
            )*

            /// Creates an empty flags set.
            #[inline]
            pub const fn empty() -> Self {
                Self(<$vk_type>::empty())
            }

            /// Checks if any flag is set.
            #[inline]
            pub const fn is_empty(&self) -> bool {
                self.0.is_empty()
            }

            /// Checks if a specific flag is set.
            #[inline]
            pub const fn contains(&self, other: Self) -> bool {
                self.0.contains(other.0)
            }

            #[inline]
            pub fn union(&self, other: Self) -> Self {
                Self(self.0 | other.0)
            }

            /// Converts from Vulkan flags.
            #[inline]
            pub const fn from_vk(flags: $vk_type) -> Self {
                Self(flags)
            }

            /// Converts to Vulkan flags.
            #[inline]
            pub const fn to_vk(self) -> $vk_type {
                self.0
            }
        }

        impl Default for $ident {
            fn default() -> Self {
                Self::empty()
            }
        }

        impl std::ops::BitOr for $ident {
            type Output = Self;

            #[inline]
            fn bitor(self, rhs: Self) -> Self::Output {
                Self(self.0 | rhs.0)
            }
        }

        impl std::ops::BitOrAssign for $ident {
            #[inline]
            fn bitor_assign(&mut self, rhs: Self) {
                self.0 |= rhs.0;
            }
        }

        impl std::ops::BitAnd for $ident {
            type Output = Self;

            #[inline]
            fn bitand(self, rhs: Self) -> Self::Output {
                Self(self.0 & rhs.0)
            }
        }

        impl std::ops::BitAndAssign for $ident {
            #[inline]
            fn bitand_assign(&mut self, rhs: Self) {
                self.0 &= rhs.0;
            }
        }

        impl std::ops::Not for $ident {
            type Output = Self;

            #[inline]
            fn not(self) -> Self::Output {
                Self(!self.0)
            }
        }
    };
}
