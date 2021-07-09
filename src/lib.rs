pub mod cache;
pub mod checksum;
pub mod cli;
pub mod markdown;
pub mod source;
pub mod stamp;

/// A resource for any stage.
pub trait Resource: checksum::Digest {
    type Id;

    /// The unique identifier. Use in combination with the checksum to ensure the resource is exactly the same.
    fn id(&self) -> &Self::Id;
}

/// A resource set for any stage.
pub trait ResourceSet: IntoIterator {}
