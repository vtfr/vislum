use ash::vk;

#[derive(Copy, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl std::fmt::Debug for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Version({})", self)
    }
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.patch)
    }
}

impl Version {
    pub const V1_0: Self = Self::new(1, 0, 0);
    pub const V1_1: Self = Self::new(1, 1, 0);
    pub const V1_2: Self = Self::new(1, 2, 0);
    pub const V1_3: Self = Self::new(1, 3, 0);

    pub const fn new(major: u32, minor: u32, patch: u32) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }

    pub const fn to_vk(&self) -> u32 {
        vk::make_api_version(0, self.major, self.minor, self.patch)
    }

    pub const fn from_vk(version: u32) -> Self {
        let major = vk::api_version_major(version);
        let minor = vk::api_version_minor(version);
        let patch = vk::api_version_patch(version);
        Self::new(major, minor, patch)
    }
}
