use std::fmt::Display;

use ash::{prelude::VkResult, vk};

#[macro_export]
macro_rules! new_extensions_struct {
    (
        $(#[$meta:meta])*
        $vis:vis struct $ident:ident {
            $(
                $(#[$field_meta:meta])*
                $field:ident => $extension_name:expr
            ),*
            $(,)?
        }
    ) => {
        #[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
        $(#[$meta])*
        $vis struct $ident {
            $(
                $(#[$field_meta])*
                pub $field: bool,
            )*
        }

        impl std::fmt::Display for $ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let entries = self.to_vk_extension_names()
                    .map(|name| name.to_str().unwrap());

                f.debug_list().entries(entries).finish()
            }
        }

        impl $ident {
            pub fn from_vk_extension_names<'a>(extension_names: impl IntoIterator<Item = &'a std::ffi::CStr>) -> Self {
                let mut extensions = Self::default();

                for extension_name in extension_names {
                    $(
                        if extension_name == $extension_name {
                            extensions.$field = true;
                            continue;
                        }
                    )*
                }

                extensions
            }

            pub fn to_vk_extension_names(&self) -> impl Iterator<Item = &'static std::ffi::CStr> {
                [
                    $(
                        ($extension_name, self.$field),
                    )*
                ]
                .into_iter()
                .filter_map(|(name, enabled)| enabled.then_some(name))
            }

            #[doc = concat!("Returns whether the [`", stringify!($ident), "`] is empty.")]
            pub fn is_empty(&self) -> bool {
                self == &Self::default()
            }

            #[doc = concat!("Compares with another [`", stringify!($ident), "`] and returns the missing extensions.")]
            pub fn difference(&self, other: &Self) -> Self {
                let mut difference = Self::default();
                
                $(
                    if other.$field && !self.$field {
                        difference.$field = true;
                    }
                )*

                difference
            }
        }
    };
}

/// Perform a two-pass call to a delegate to read into a vector.
/// 
/// First call reads the number of elements to allocate.
/// Second call reads the elements into the vector.
pub(in crate::rhi) unsafe fn read_into_vec<T, F>(mut delegate: F) -> ash::prelude::VkResult<Vec<T>> 
where 
    F: FnMut(*mut u32, *mut T) -> vk::Result,
    T: Copy,
{
    let mut count = 0;

    delegate(&mut count, std::ptr::null_mut())
        .result()?;

    let mut vec = Vec::with_capacity(count as usize);

    // Defer processing the result until after the vector is set the appropriate length
    let result = delegate(&mut count, vec.as_mut_ptr());

    unsafe { vec.set_len(count as usize); }

    result.result().map(|_| vec)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Version {
    pub const VERSION_1_0: Self = Self::new(1, 0, 0);
    pub const VERSION_1_1: Self = Self::new(1, 1, 0);
    pub const VERSION_1_2: Self = Self::new(1, 2, 0);
    pub const VERSION_1_3: Self = Self::new(1, 3, 0);

    #[inline]
    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self { major, minor, patch }
    }

    #[inline]
    pub const fn from_vk(version: u32) -> Self {
        Self {
            major: vk::api_version_major(version),
            minor: vk::api_version_minor(version),
            patch: vk::api_version_patch(version),
        }
    }

    #[inline]
    pub const fn to_vk(&self) -> u32 {
        vk::make_api_version(0, self.major, self.minor, self.patch)
    }

}