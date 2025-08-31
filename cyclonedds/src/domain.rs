//! A Domain defines the boundary of a shared data-space (identified by a 32-bit
//! domain ID). Only entities within the same domain can publish or subscribe to
//! each other's data.

use crate::internal::ffi;
use crate::{Error, Result};

/// A communication boundary for DDS publish-subscribe traffic.
#[derive(Debug)]
pub struct Domain {
    pub(crate) id: cyclonedds_sys::dds_domainid_t,
    pub(crate) inner: cyclonedds_sys::dds_entity_t,
}

impl Domain {
    /// Creates a new domain with the given `domain_id` using the default
    /// Cyclone DDS configuration.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if Cyclone DDS fails to create the domain, for
    /// example if the `domain_id` is out of range or already in use.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::Domain;
    /// // Create a default domain.
    /// let default_domain = Domain::default();
    ///
    /// // Create a new domain with the domain ID of 1.
    /// let domain = Domain::new(1)?;
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn new(domain_id: u32) -> Result<Self> {
        let inner = ffi::dds_create_domain(domain_id)?;
        Ok(Self {
            id: domain_id,
            inner,
        })
    }

    /// Creates a new domain with the given `domain_id` and an XML `config`
    /// string passed directly to Cyclone DDS.
    ///
    /// The `config` must be a valid Cyclone DDS XML configuration fragment. See
    /// the [Cyclone DDS
    /// documentation](https://github.com/eclipse-cyclonedds/cyclonedds/blob/master/docs/manual/options.md)
    /// for the full set of configuration options.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the `config` contains interior null bytes or if
    /// the domain cannot be created with the provided domain ID and
    /// configuration.
    ///
    /// # Examples
    ///
    /// ```
    /// use cyclonedds::Domain;
    ///
    /// let config = r#"
    ///     <Domain>
    ///         <Tracing>
    ///             <Verbosity>warning</Verbosity>
    ///             <OutputFile>stderr</OutputFile>
    ///         </Tracing>
    ///     </Domain>"#;
    ///
    /// // Create a new domain with the configuration and a domain ID of 1.
    /// let domain = Domain::new_with_xml_config(1, config)?;
    /// # Ok::<_, cyclonedds::Error>(())
    /// ```
    pub fn new_with_xml_config(domain_id: u32, config: &str) -> Result<Self> {
        let config = std::ffi::CString::new(config).map_err(|_err| Error::BadParameter)?;
        let inner = ffi::dds_create_domain_with_config(domain_id, &config)?;
        Ok(Self {
            id: domain_id,
            inner,
        })
    }
}

impl Drop for Domain {
    fn drop(&mut self) {
        if self.inner != 0 {
            let result = ffi::dds_delete(self.inner);
            debug_assert!(
                result.is_ok(),
                "unable to delete {self:?}: failed with {result:?}"
            );
        }
    }
}

impl Default for Domain {
    fn default() -> Self {
        Self {
            id: cyclonedds_sys::DOMAIN_DEFAULT,
            inner: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_create() {
        let domain_id = crate::tests::domain::unique_id();
        Domain::new(domain_id).unwrap();
    }

    #[test]
    fn test_domain_create_default() {
        let domain = Domain::default();
        assert_eq!(domain.id, cyclonedds_sys::DOMAIN_DEFAULT);
    }

    #[test]
    fn test_domain_create_with_explicit_default_id() {
        let domain_id = cyclonedds_sys::DOMAIN_DEFAULT;
        let domain = Domain::new(domain_id).unwrap_err();
        assert_eq!(domain, Error::BadParameter);
    }

    #[test]
    fn test_domain_create_with_empty_xml_config() {
        let domain_id = crate::tests::domain::unique_id();
        let xml_config = "";
        Domain::new_with_xml_config(domain_id, xml_config).unwrap();
    }

    #[test]
    fn test_domain_create_with_xml_config_with_invalid_string() {
        let domain_id = crate::tests::domain::unique_id();
        let xml_config = "\0";
        let domain = Domain::new_with_xml_config(domain_id, xml_config).unwrap_err();
        assert_eq!(domain, Error::BadParameter);
    }

    #[test]
    fn test_domain_create_with_xml_config_with_explicit_default_id() {
        let domain_id = cyclonedds_sys::DOMAIN_DEFAULT;
        let xml_config = "";
        let domain = Domain::new_with_xml_config(domain_id, xml_config).unwrap_err();
        assert_eq!(domain, Error::BadParameter);
    }

    #[test]
    fn test_domain_create_with_xml_config_with_malformed_xml() {
        let domain_id = crate::tests::domain::unique_id();
        let xml_config = "<>";
        let domain = Domain::new_with_xml_config(domain_id, xml_config).unwrap_err();
        assert_eq!(domain, Error::NonSpecific);
    }

    #[test]
    fn test_domain_create_with_xml_config_with_valid_xml() {
        let domain_id = crate::tests::domain::unique_id();
        let xml_config = "<Domain><General><MaxMessageSize>1400B</MaxMessageSize></General>";
        Domain::new_with_xml_config(domain_id, xml_config).unwrap();
    }
}
