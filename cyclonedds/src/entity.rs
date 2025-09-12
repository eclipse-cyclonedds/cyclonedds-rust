//! The base of the DDS entity hierarchy.

use crate::internal::ffi;
use crate::{Result, Status};

/// A unique opaque handle identifying an instance.
///
/// For keyed topics this corresponds to a specific key value, but applications
/// should treat it as an opaque DDS handle.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct InstanceHandle {
    pub(crate) inner: cyclonedds_sys::dds_instance_handle_t,
}

/// A raw entity ID for an entity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct EntityId {
    pub(crate) inner: cyclonedds_sys::dds_entity_t,
}

// TODO should the Entity trait be sealed?
/// Common interface implemented by all members of the DDS entity hierarchy.
pub trait Entity {
    /// Returns the [`EntityId`] of this entity.
    fn id(&self) -> EntityId;

    /// Returns the [`InstanceHandle`] of this entity.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`](crate::Error) specifying the reason if the instance
    /// handle fails to be retrieved.
    fn instance_handle(&self) -> Result<InstanceHandle> {
        let entity = self.id();
        let inner = ffi::dds_get_instance_handle(entity.inner)?;
        Ok(InstanceHandle { inner })
    }

    fn status_changes(&self) -> Result<Status> {
        let entity = self.id();
        let status = ffi::dds_get_status_changes(entity.inner)?;
        Status::from_bits(status).ok_or(crate::error::Error::BadParameter)
    }

    fn take_status(&self, mask: Option<Status>) -> Result<Status> {
        let entity = self.id();
        let mask = mask.unwrap_or(Status::all()).bits();
        let status = ffi::dds_take_status(entity.inner, mask)?;
        Status::from_bits(status).ok_or(crate::error::Error::BadParameter)
    }

    /// Reads the status flags matching `mask` without clearing them, or all
    /// flags if `mask` is `None`.
    ///
    /// # Errors
    ///
    /// - Returns an [`Error`](crate::Error) if the status bits of the
    ///   corresponding entity could not be retrieved (e.g. the entity no longer
    ///   exists).
    fn read_status(&self, mask: Option<Status>) -> Result<Status> {
        let entity = self.id();
        let mask = mask.unwrap_or(Status::all()).bits();
        let status = ffi::dds_read_status(entity.inner, mask)?;
        Status::from_bits(status).ok_or(crate::error::Error::BadParameter)
    }

    /// Returns the status mask enabled on the entity.
    ///
    /// # Errors
    ///
    /// - Returns an [`Error`](crate::Error) if the status mask of the
    ///   corresponding entity could not be retrieved (e.g. the entity no longer
    ///   exists).
    ///
    /// - Returns [`BadParameter`](crate::Error::BadParameter) if the retrieved
    ///   bits do not correspond to a valid [`Status`].
    ///
    fn status_mask(&self) -> Result<Status> {
        let entity = self.id();
        let mask = ffi::dds_get_status_mask(entity.inner)?;
        Status::from_bits(mask).ok_or(crate::error::Error::BadParameter)
    }

    /// Sets and enables a status mask on the entity.
    ///
    /// Only status flags included in `mask` will trigger listener callbacks or
    /// be reported via [`status_changes`](Entity::status_changes).
    ///
    /// # Errors
    ///
    /// - Returns an [`Error`](crate::Error) if the status mask of the
    ///   corresponding entity could not be set (e.g. the entity no longer
    ///   exists).
    fn set_status_mask(&self, mask: Status) -> Result<()> {
        let entity = self.id();
        let mask = mask.bits();
        ffi::dds_set_status_mask(entity.inner, mask)
    }
}

macro_rules! impl_entity {
    ($ty:ty) => {
        impl Entity for $ty {
            fn id(&self) -> EntityId {
                EntityId { inner: self.inner }
            }
        }
    };
    ($ty:ty where $($bounds:tt)*) => {
        impl<$($bounds)*> Entity for $ty {
            fn id(&self) -> EntityId {
                EntityId { inner: self.inner }
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entity_id_all_entity_types() {}
}
