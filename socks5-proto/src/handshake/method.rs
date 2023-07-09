#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct Method(pub u8);

impl Method {
    pub const NONE: Self = Self(0x00);
    pub const GSSAPI: Self = Self(0x01);
    pub const PASSWORD: Self = Self(0x02);
    pub const UNACCEPTABLE: Self = Self(0xff);
}

impl From<u8> for Method {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<Method> for u8 {
    fn from(value: Method) -> Self {
        value.0
    }
}
