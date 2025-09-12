//! Safe and ergonomic wrappers for the functionality from [`cyclonedds_sys`].
//!
//! This module is exposed to make it easier for applications to poke at the
//! low-level implementation details of Cyclone DDS.

#![allow(unsafe_code)]

use crate::Result;
use crate::error::IntoError;

pub fn dds_get_instance_handle(
    entity: cyclonedds_sys::dds_entity_t,
) -> Result<cyclonedds_sys::dds_instance_handle_t> {
    let mut handle = 0;
    unsafe { cyclonedds_sys::dds_get_instance_handle(entity, &raw mut handle) }.into_error()?;
    Ok(handle)
}

pub fn dds_get_status_changes(entity: cyclonedds_sys::dds_entity_t) -> Result<u32> {
    let mut status = 0;
    unsafe { cyclonedds_sys::dds_get_status_changes(entity, &raw mut status) }.into_error()?;
    Ok(status)
}

pub fn dds_read_status(entity: cyclonedds_sys::dds_entity_t, mask: u32) -> Result<u32> {
    let mut status: u32 = 0;
    unsafe { cyclonedds_sys::dds_read_status(entity, &raw mut status, mask) }.into_error()?;
    Ok(status)
}

pub fn dds_take_status(entity: cyclonedds_sys::dds_entity_t, mask: u32) -> Result<u32> {
    let mut status: u32 = 0;
    unsafe { cyclonedds_sys::dds_take_status(entity, &raw mut status, mask) }.into_error()?;
    Ok(status)
}

pub fn dds_get_status_mask(entity: cyclonedds_sys::dds_entity_t) -> Result<u32> {
    let mut mask: u32 = 0;
    unsafe { cyclonedds_sys::dds_get_status_mask(entity, &raw mut mask) }.into_error()?;
    Ok(mask)
}

pub fn dds_set_status_mask(entity: cyclonedds_sys::dds_entity_t, mask: u32) -> Result<()> {
    unsafe { cyclonedds_sys::dds_set_status_mask(entity, mask) }.into_error()?;
    Ok(())
}
