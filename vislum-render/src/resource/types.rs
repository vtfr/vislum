#[macro_export]
macro_rules! wrap_wgpu {
    {
        $(#[$meta:meta])*
        $vis:vis struct $name:ident: $inner:path;
    } => {
        $(#[$meta])*
        #[derive(Clone)]
        $vis struct $name(std::sync::Arc<$inner>);

        impl $name {
            #[doc = concat!("Creates a new ", stringify!($name))]
            pub fn new(inner: $inner) -> Self {
                $name(std::sync::Arc::new(inner))
            }

            #[doc = concat!("Creates a new ", stringify!($name), " from an existing Arc")]
            pub fn new_arc(inner: std::sync::Arc<$inner>) -> Self {
                $name(inner)
            }
        }
        
        impl std::convert::From<$inner> for $name {
            fn from(inner: $inner) -> Self {
                $name::new(inner)
            }
        }

        impl std::convert::From<std::sync::Arc<$inner>> for $name {
            fn from(inner: std::sync::Arc<$inner>) -> Self {
                $name::new_arc(inner)
            }
        }

        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(stringify!($name))
            }
        }

        impl std::ops::Deref for $name {
            type Target = $inner;

            fn deref(&self) -> &Self::Target {
                &*self.0
            }
        }
    };
}

wrap_wgpu! {
    /// A device.
    pub struct RenderDevice: wgpu::Device;
}

wrap_wgpu! {
    /// A queue.
    pub struct RenderQueue: wgpu::Queue;
}

#[macro_export]
macro_rules! create_atomic_id {
    {
        $(#[$meta:meta])*
        $vis:vis struct $name:ident;
    } => {
        $(#[$meta])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        $vis struct $name(u64);

        impl $name {
            pub fn new() -> Self {
                static ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

                let id = ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                Self(id)
            }
        }
    };
}

#[macro_export]
macro_rules! wrap_wgpu_with_atomic_id {
    {
        $(#[$meta:meta])*
        $vis:vis struct $name:ident($id:ident): $inner:path;
    } => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $id(u64);

        $(#[$meta])*
        #[derive(Clone)]
        $vis struct $name {
            id: $id,
            inner: std::sync::Arc<$inner>
        }

        impl $name {
            #[doc = concat!("Creates a new ", stringify!($name))]
            pub fn new(inner: $inner) -> Self {
                static ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

                let id = ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

                $name {
                    id: $id(id),
                    inner: std::sync::Arc::new(inner)
                }
            }

            pub fn id(&self) -> $id {
                self.id
            }
        }
        
        impl std::fmt::Debug for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(stringify!($name))
            }
        }

        impl std::ops::Deref for $name {
            type Target = $inner;

            fn deref(&self) -> &Self::Target {
                &*self.inner
            }
        }
    };
}

// Re-export wgpu types.
pub mod wgpu {
    pub use wgpu::*;
}

// Re-export macros
pub use crate::wrap_wgpu_with_atomic_id;
pub use crate::create_atomic_id;