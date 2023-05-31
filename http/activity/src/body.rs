#[derive(Debug)]
#[cfg_attr(feature = "serde", derive(serde::Deserialize, serde::Serialize))]
#[cfg_attr(feature = "rkyv", derive(rkyv::Archive, rkyv::Deserialize, rkyv::Serialize))]
#[cfg_attr(feature = "rkyv", archive(check_bytes))]
pub enum HttpBody {
    None,
    Text(String),
    #[cfg(feature = "rkyv")]
    Bytes(Vec<u8>),
    #[cfg(not(feature = "rkyv"))]
    Bytes(bytes::Bytes),
}