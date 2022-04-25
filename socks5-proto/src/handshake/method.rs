#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct HandshakeMethod(pub u8);

#[allow(non_upper_case_globals)]
impl HandshakeMethod {
    pub const None: Self = Self(0x00);
    pub const Gssapi: Self = Self(0x01);
    pub const Password: Self = Self(0x02);
    pub const Unacceptable: Self = Self(0xff);
}

impl From<u8> for HandshakeMethod {
    fn from(value: u8) -> Self {
        Self(value)
    }
}

impl From<HandshakeMethod> for u8 {
    fn from(value: HandshakeMethod) -> Self {
        value.0
    }
}
