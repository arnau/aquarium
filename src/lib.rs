pub mod cache;
pub mod checksum;
pub mod cli;
pub mod markdown;
pub mod resource_type;
pub mod source;
pub mod stamp;
pub mod zola;

pub use cache::Cache;

/// A resource for any stage.
pub trait Resource: checksum::Digest {
    type Id;

    /// The unique identifier. Use in combination with the checksum to ensure the resource is exactly the same.
    fn id(&self) -> &Self::Id;
}

/// A resource set for any stage.
pub trait ResourceSet: IntoIterator {}

#[allow(dead_code)]
pub(crate) fn to_hex(buffer: &[u8]) -> String {
    let mut s = String::new();
    let table = b"0123456789abcdef";

    for &b in buffer {
        s.push(table[(b >> 4) as usize] as char);
        s.push(table[(b & 0xf) as usize] as char);
    }

    s
}
