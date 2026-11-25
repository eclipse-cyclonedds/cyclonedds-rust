//! A minimal PSMX plugin, used to exercise the shared-memory receive path of
//! the Rust binding in tests.
//!
//! Cyclone DDS off-loads data transfer to a Pub/Sub Message Exchange (PSMX)
//! plugin, loaded with `dlopen` from the `PubSubMessageExchange` element of the
//! domain's XML configuration. Shared memory is one such plugin
//! (iceoryx-backed); this one instead hands the loaned sample straight to the
//! reader in the same process, which exercises exactly the same Cyclone DDS
//! machinery — `request_loan`, the PSMX write path, and the
//! `ddsi_serdata_from_psmx` op on the receiving side — without needing a
//! shared-memory daemon in CI.
//!
//! It implements version 1 of the PSMX interface (`create_topic` and `deinit`
//! left null, `create_topic_with_type` and `delete_psmx` set), so Cyclone DDS
//! fills in everything but the `ops` tables.
//!
//! # Usage
//!
//! Point a domain's configuration at the built library. The `type` attribute
//! determines the constructor symbol that is looked up
//! (`<type>_create_psmx`), and `library` overrides the default
//! `psmx_<type>` file name:
//!
//! ```xml
//! <Domain id="0">
//!   <General>
//!     <Interfaces>
//!       <PubSubMessageExchange type="rusttest"
//!                              library="/path/to/libpsmx_rusttest.so"
//!                              config="LOCATOR=0101..;INSTANCE_NAME=shm;"/>
//!     </Interfaces>
//!   </General>
//! </Domain>
//! ```
//!
//! Two domains that share a PSMX Locator will exchange data through this
//! plugin.
//!
//! # Scope
//!
//! This is a test fixture, not a usable transport: delivery is in-process only,
//! there is no cross-process shared memory, and `take` is unimplemented (Cyclone
//! DDS never calls it). It reports the shared-memory feature so that
//! `dds_is_shared_memory_available` returns true for endpoints using it.

// This crate is a C ABI plugin, so it is unsafe throughout and its exported
// items are named to match what Cyclone DDS looks up rather than to read well
// as Rust.
#![allow(unsafe_code)]

use std::ffi::{CStr, c_char, c_void};
use std::sync::Mutex;

/// `DDS_RETCODE_OK` as a [`dds_return_t`][cyclonedds_sys::dds_return_t].
///
/// The generated binding types it as unsigned, because it is zero, while every
/// operation returns the signed `dds_return_t`. The error codes are already
/// signed and so need no equivalent.
const RETCODE_OK: cyclonedds_sys::dds_return_t =
    cyclonedds_sys::DDS_RETCODE_OK as cyclonedds_sys::dds_return_t;

/// The most PSMX Readers this fixture will track at once.
const MAX_READERS: usize = 64;

/// PSMX Readers currently registered, for routing published samples.
///
/// A PSMX Reader is registered when Cyclone DDS asks for asynchronous delivery
/// through `on_data_available`, which is also where it tells us the DDS Reader
/// to deliver to.
///
/// Deliberately a fixed-size array rather than a map: Cyclone DDS `dlclose`s this
/// plugin when the last domain using it goes away, which unmaps the statics
/// below. Anything still reachable only from here at that point is gone for good
/// and LeakSanitizer rightly calls it a leak, so the simplest way to stay honest
/// is to never allocate from a `static` at all.
static READERS: Mutex<[Registration; MAX_READERS]> =
    Mutex::new([Registration::VACANT; MAX_READERS]);

/// A slot in [`READERS`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Registration {
    /// Address of the owning [`Endpoint`], or zero when the slot is free. Also
    /// how the slot is found again to release it.
    endpoint: usize,
    /// The DDS Reader entity to deliver samples to.
    entity: cyclonedds_sys::dds_entity_t,
}

impl Registration {
    /// An unused slot.
    const VACANT: Self = Self {
        endpoint: 0,
        entity: 0,
    };
}

/// A PSMX Instance.
///
/// Cyclone DDS is handed a pointer to the embedded `dds_psmx`, so that must stay
/// the first field.
#[repr(C)]
struct Psmx {
    inner: cyclonedds_sys::dds_psmx,
    /// Where to record what crossed this interface, from the `TRACE` option of
    /// the `config` attribute. Lets a test assert on the transport itself —
    /// whether a sample arrived at all, and whether it was raw or serialized —
    /// rather than on instrumentation inside the binding.
    trace: Option<std::path::PathBuf>,
}

/// A PSMX Topic.
#[repr(C)]
struct Topic {
    inner: cyclonedds_sys::dds_psmx_topic,
    /// Owned copy of the topic name, used to route samples to readers.
    name: String,
}

/// A PSMX Endpoint, either a reader or a writer.
#[repr(C)]
struct Endpoint {
    inner: cyclonedds_sys::dds_psmx_endpoint,
    /// Owned copy of the topic name, so delivery does not have to walk back
    /// through the C structures.
    topic_name: String,
}

/// A loaned sample owned by this plugin.
///
/// Cyclone DDS is handed a pointer to the embedded `dds_loaned_sample`, whose
/// `metadata` and `sample_ptr` point into the other two fields. Both are
/// separately boxed so the pointers stay valid when the `Loan` itself moves.
#[repr(C)]
struct Loan {
    inner: cyclonedds_sys::dds_loaned_sample,
    metadata: Box<cyclonedds_sys::dds_psmx_metadata>,
    /// The payload. `u64`-backed so the buffer is aligned well enough to hold
    /// any type the application might store in it, as the interface requires.
    payload: Box<[u64]>,
}

/// Produce an all-zero instance of a plain-old-data C structure.
///
/// Every structure this is used for consists solely of integers, C enums, raw
/// pointers, and optional function pointers, for all of which an all-zero bit
/// pattern is a valid value.
fn zeroed<T>() -> T {
    // SAFETY: see above; only used for POD C structures.
    unsafe { std::mem::MaybeUninit::zeroed().assume_init() }
}

/// Read one `KEY=VALUE` option out of a PSMX configuration string.
///
/// ## Safety
/// `config` must be null or a valid configuration string as Cyclone DDS builds
/// them.
unsafe fn config_option(config: *const c_char, key: &CStr) -> Option<String> {
    if config.is_null() {
        return None;
    }

    // SAFETY: guaranteed by the caller. Cyclone DDS allocates the result, so it
    // goes back through `dds_free`.
    let value = unsafe { cyclonedds_sys::dds_psmx_get_config_option_value(config, key.as_ptr()) };
    if value.is_null() {
        return None;
    }

    // SAFETY: a non-null return is a valid null-terminated string.
    let owned = unsafe { CStr::from_ptr(value) }
        .to_string_lossy()
        .into_owned();
    // SAFETY: allocated by Cyclone DDS for us to release.
    unsafe { cyclonedds_sys::dds_free(value.cast()) };

    Some(owned)
}

/// Record that a sample crossed this interface, for a test to assert on.
///
/// One line per published sample: the topic name, then the state Cyclone DDS put
/// in the loan — which is what distinguishes a zero-copy transfer (`RAW_DATA`)
/// from a serialized one.
fn trace_write(psmx: &Psmx, topic_name: &str, state: cyclonedds_sys::dds_loaned_sample_state_t) {
    let Some(path) = psmx.trace.as_ref() else {
        return;
    };

    let state = match state {
        cyclonedds_sys::dds_loaned_sample_state_DDS_LOANED_SAMPLE_STATE_RAW_DATA => "RAW_DATA",
        cyclonedds_sys::dds_loaned_sample_state_DDS_LOANED_SAMPLE_STATE_RAW_KEY => "RAW_KEY",
        cyclonedds_sys::dds_loaned_sample_state_DDS_LOANED_SAMPLE_STATE_SERIALIZED_DATA => {
            "SERIALIZED_DATA"
        }
        cyclonedds_sys::dds_loaned_sample_state_DDS_LOANED_SAMPLE_STATE_SERIALIZED_KEY => {
            "SERIALIZED_KEY"
        }
        _ => "UNINITIALIZED",
    };

    use std::io::Write as _;
    if let Ok(mut file) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
    {
        let _ = writeln!(file, "{topic_name} {state}");
    }
}

/// Release a loan once its reference count reaches zero.
///
/// ## Safety
/// `loan` must be a pointer previously returned by [`request_loan`] whose
/// reference count has reached zero.
unsafe extern "C" fn free_loan(loan: *mut cyclonedds_sys::dds_loaned_sample) {
    if loan.is_null() {
        return;
    }

    // SAFETY: guaranteed by the caller, this came from `Box::into_raw` in
    // `request_loan`, where the `dds_loaned_sample` is the first field.
    drop(unsafe { Box::from_raw(loan.cast::<Loan>()) });
}

/// Borrow memory for a sample of `size_requested` bytes.
///
/// The plugin owns the `dds_loaned_sample`, its metadata, and the payload.
/// Cyclone DDS fills in `loan_origin` and `refc` after a successful return.
///
/// ## Safety
/// `endpoint` must be a PSMX Writer created by [`create_endpoint`].
unsafe extern "C" fn request_loan(
    _endpoint: *mut cyclonedds_sys::dds_psmx_endpoint,
    size_requested: u32,
) -> *mut cyclonedds_sys::dds_loaned_sample {
    // Round up so the `u64` backing store covers the requested byte count.
    let words = (size_requested as usize)
        .div_ceil(std::mem::size_of::<u64>())
        .max(1);

    let mut loan = Box::new(Loan {
        inner: zeroed(),
        metadata: Box::new(zeroed()),
        payload: vec![0_u64; words].into_boxed_slice(),
    });

    loan.inner.ops.free = Some(free_loan);
    loan.inner.metadata = &raw mut *loan.metadata;
    loan.inner.sample_ptr = loan.payload.as_mut_ptr().cast::<c_void>();

    Box::into_raw(loan).cast()
}

/// Publish a loaned sample to every reader registered for this topic.
///
/// Uses `dds_reader_store_loaned_sample_wr_metadata` rather than
/// `dds_reader_store_loaned_sample` so that delivery does not depend on the
/// writer having been discovered in the reader's domain. That keeps the test
/// deterministic: PSMX becomes the only way a sample can reach the reader.
///
/// The loan is not consumed; Cyclone DDS releases it after this returns.
///
/// ## Safety
/// - `endpoint` must be a PSMX Writer created by [`create_endpoint`].
/// - `loan` must be a fully-initialized loan obtained from [`request_loan`], with a sample state
///   other than uninitialized.
unsafe extern "C" fn write_loan(
    endpoint: *mut cyclonedds_sys::dds_psmx_endpoint,
    loan: *mut cyclonedds_sys::dds_loaned_sample,
) -> cyclonedds_sys::dds_return_t {
    if endpoint.is_null() || loan.is_null() {
        return cyclonedds_sys::DDS_RETCODE_BAD_PARAMETER;
    }

    // SAFETY: guaranteed by the caller, `dds_psmx_endpoint` is the first field
    // of `Endpoint`.
    let endpoint = unsafe { &*endpoint.cast::<Endpoint>() };

    // Record what crossed the interface before delivering, so a test can assert
    // on the transport rather than on the binding's internals.
    //
    // SAFETY: for interface version 1 Cyclone DDS fills in `psmx_topic`, and a
    // topic's `psmx_instance` is the instance this plugin created.
    unsafe {
        let topic = &*endpoint.inner.psmx_topic.cast::<Topic>();
        let psmx = &*topic.inner.psmx_instance.cast::<Psmx>();
        trace_write(psmx, &endpoint.topic_name, (*(*loan).metadata).sample_state);
    }

    // Collect the matching readers under the lock, then release it before
    // delivering: `dds_reader_store_loaned_sample_wr_metadata` re-enters Cyclone
    // DDS, which may call back into this plugin.
    let matched: Vec<cyclonedds_sys::dds_entity_t> = {
        let registry = READERS.lock().expect("PSMX reader registry poisoned");
        registry
            .iter()
            .filter(|slot| **slot != Registration::VACANT)
            // SAFETY: a registered slot addresses an `Endpoint` that is alive
            // until `delete_endpoint` clears it, which takes this same lock.
            .filter(|slot| {
                unsafe { &*(slot.endpoint as *const Endpoint) }.topic_name == endpoint.topic_name
            })
            .map(|slot| slot.entity)
            .collect()
    };

    for reader in matched {
        // SAFETY: `loan` is a valid, fully-initialized loan, and `reader.entity`
        // was handed to us by Cyclone DDS for asynchronous delivery. The call
        // takes its own reference if it retains the loan.
        let result = unsafe {
            cyclonedds_sys::dds_reader_store_loaned_sample_wr_metadata(
                reader,
                loan,
                0,
                false,
                cyclonedds_sys::DURATION_INFINITE,
            )
        };

        if result != RETCODE_OK {
            return result;
        }
    }

    RETCODE_OK
}

/// Take an unread sample from a PSMX Reader.
///
/// Never called by Cyclone DDS, and unnecessary here because [`write_loan`]
/// delivers synchronously.
///
/// ## Safety
/// Sound for any argument; the argument is not dereferenced.
unsafe extern "C" fn take(
    _endpoint: *mut cyclonedds_sys::dds_psmx_endpoint,
) -> *mut cyclonedds_sys::dds_loaned_sample {
    std::ptr::null_mut()
}

/// Register a DDS Reader for asynchronous delivery of samples arriving at a PSMX
/// Reader.
///
/// ## Safety
/// `endpoint` must be a PSMX Reader created by [`create_endpoint`].
unsafe extern "C" fn on_data_available(
    endpoint: *mut cyclonedds_sys::dds_psmx_endpoint,
    reader: cyclonedds_sys::dds_entity_t,
) -> cyclonedds_sys::dds_return_t {
    if endpoint.is_null() {
        return cyclonedds_sys::DDS_RETCODE_BAD_PARAMETER;
    }

    let mut registry = READERS.lock().expect("PSMX reader registry poisoned");
    let Some(slot) = registry
        .iter_mut()
        .find(|slot| **slot == Registration::VACANT)
    else {
        panic!("the PSMX test fixture tracks at most {MAX_READERS} readers at a time");
    };

    *slot = Registration {
        endpoint: endpoint as usize,
        entity: reader,
    };

    RETCODE_OK
}

/// Create a PSMX Endpoint for a PSMX Topic.
///
/// Only `ops` is set; Cyclone DDS initializes the remaining fields for
/// interface version 1.
///
/// ## Safety
/// `topic` must be a PSMX Topic created by [`create_topic_with_type`].
unsafe extern "C" fn create_endpoint(
    topic: *mut cyclonedds_sys::dds_psmx_topic,
    _qos: *const cyclonedds_sys::dds_qos_t,
    _endpoint_type: cyclonedds_sys::dds_psmx_endpoint_type_t,
) -> *mut cyclonedds_sys::dds_psmx_endpoint {
    if topic.is_null() {
        return std::ptr::null_mut();
    }

    // SAFETY: guaranteed by the caller, `dds_psmx_topic` is the first field of
    // `Topic`.
    let topic_ref = unsafe { &*topic.cast::<Topic>() };

    let mut endpoint = Box::new(Endpoint {
        inner: zeroed(),
        topic_name: topic_ref.name.clone(),
    });

    endpoint.inner.ops = cyclonedds_sys::dds_psmx_endpoint_ops_t {
        request_loan: Some(request_loan),
        write: Some(write_loan),
        take: Some(take),
        on_data_available: Some(on_data_available),
        // Leaving this null keeps `wants_key` false, so Cyclone DDS never asks
        // for a serialized key blob.
        write_with_key: None,
    };

    Box::into_raw(endpoint).cast()
}

/// Destroy a PSMX Endpoint, deregistering it if it was a reader.
///
/// ## Safety
/// `endpoint` must be a PSMX Endpoint created by [`create_endpoint`] on which no
/// further operations will be invoked.
unsafe extern "C" fn delete_endpoint(
    endpoint: *mut cyclonedds_sys::dds_psmx_endpoint,
) -> cyclonedds_sys::dds_return_t {
    if endpoint.is_null() {
        return cyclonedds_sys::DDS_RETCODE_BAD_PARAMETER;
    }

    // SAFETY: guaranteed by the caller, this came from `Box::into_raw` in
    // `create_endpoint`.
    let endpoint = unsafe { Box::from_raw(endpoint.cast::<Endpoint>()) };

    let address = (&raw const *endpoint) as usize;

    {
        let mut registry = READERS.lock().expect("PSMX reader registry poisoned");
        for slot in registry.iter_mut() {
            if slot.endpoint == address {
                *slot = Registration::VACANT;
            }
        }
    }

    drop(endpoint);

    RETCODE_OK
}

/// Create a PSMX Topic in a PSMX Instance.
///
/// Only `ops` is set; Cyclone DDS initializes the remaining fields for
/// interface version 1.
///
/// ## Safety
/// - `psmx` must be a PSMX Instance created by [`rusttest_create_psmx`].
/// - `topic_name` must be a valid null-terminated C string.
unsafe extern "C" fn create_topic_with_type(
    psmx: *mut cyclonedds_sys::dds_psmx,
    topic_name: *const c_char,
    _type_name: *const c_char,
    _data_type_props: cyclonedds_sys::dds_data_type_properties_t,
    _type_definition: *const cyclonedds_sys::ddsi_type,
    _sizeof_type: u32,
) -> *mut cyclonedds_sys::dds_psmx_topic {
    if psmx.is_null() || topic_name.is_null() {
        return std::ptr::null_mut();
    }

    // SAFETY: guaranteed by the caller.
    let name = unsafe { CStr::from_ptr(topic_name) }
        .to_string_lossy()
        .into_owned();

    let mut topic = Box::new(Topic {
        inner: zeroed(),
        name,
    });

    topic.inner.ops = cyclonedds_sys::dds_psmx_topic_ops_t {
        create_endpoint: Some(create_endpoint),
        delete_endpoint: Some(delete_endpoint),
    };

    Box::into_raw(topic).cast()
}

/// Destroy a PSMX Topic.
///
/// ## Safety
/// `topic` must be a PSMX Topic created by [`create_topic_with_type`] whose
/// endpoints have all been deleted.
unsafe extern "C" fn delete_topic(
    topic: *mut cyclonedds_sys::dds_psmx_topic,
) -> cyclonedds_sys::dds_return_t {
    if topic.is_null() {
        return cyclonedds_sys::DDS_RETCODE_BAD_PARAMETER;
    }

    // SAFETY: guaranteed by the caller, this came from `Box::into_raw` in
    // `create_topic_with_type`.
    drop(unsafe { Box::from_raw(topic.cast::<Topic>()) });

    RETCODE_OK
}

/// Report whether this plugin can carry a given type and QoS.
///
/// Accepts everything, so that both the serialized and the raw payload paths can
/// be exercised.
///
/// ## Safety
/// Sound for any arguments; none are dereferenced.
unsafe extern "C" fn type_qos_supported(
    _psmx: *mut cyclonedds_sys::dds_psmx,
    _forwhat: cyclonedds_sys::dds_psmx_endpoint_type_t,
    _data_type_props: cyclonedds_sys::dds_data_type_properties_t,
    _qos: *const cyclonedds_sys::dds_qos_t,
) -> bool {
    true
}

/// Report the identifier of the node this PSMX Instance runs on.
///
/// A fixed value, as delivery never leaves the process.
///
/// ## Safety
/// Sound for any argument; the argument is not dereferenced.
unsafe extern "C" fn get_node_id(
    _psmx: *const cyclonedds_sys::dds_psmx,
) -> cyclonedds_sys::dds_psmx_node_identifier_t {
    cyclonedds_sys::dds_psmx_node_identifier_t { x: [0x5a; 16] }
}

/// Report the features this PSMX Instance supports.
///
/// Claims shared memory, which is what makes `dds_is_shared_memory_available`
/// return true for endpoints using this plugin.
///
/// ## Safety
/// Sound for any argument; the argument is not dereferenced.
unsafe extern "C" fn supported_features(
    _psmx: *const cyclonedds_sys::dds_psmx,
) -> cyclonedds_sys::dds_psmx_features_t {
    cyclonedds_sys::DDS_PSMX_FEATURE_SHARED_MEMORY
}

/// Destroy a PSMX Instance.
///
/// ## Safety
/// `psmx` must be a PSMX Instance created by [`rusttest_create_psmx`] whose
/// topics have all been deleted.
unsafe extern "C" fn delete_psmx(psmx: *mut cyclonedds_sys::dds_psmx) {
    if psmx.is_null() {
        return;
    }

    // SAFETY: guaranteed by the caller, this came from `Box::into_raw` in
    // `rusttest_create_psmx`.
    drop(unsafe { Box::from_raw(psmx.cast::<Psmx>()) });
}

/// Construct a PSMX Instance.
///
/// This is the entry point Cyclone DDS looks up with `dlsym`, named after the
/// `type` attribute of the `PubSubMessageExchange` configuration element.
///
/// Implements interface version 1: only `ops` is set, and every other field is
/// left zeroed for Cyclone DDS to initialize. Cyclone DDS rejects the plugin if
/// `instance_id` or `instance_name` is set here.
///
/// ## Safety
/// `psmx_instance` must be a valid pointer to write the new instance through.
/// `config` must be null or a valid C string.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn rusttest_create_psmx(
    psmx_instance: *mut *mut cyclonedds_sys::dds_psmx,
    _identifier: cyclonedds_sys::dds_psmx_instance_id_t,
    config: *const c_char,
) -> cyclonedds_sys::dds_return_t {
    if psmx_instance.is_null() {
        return cyclonedds_sys::DDS_RETCODE_BAD_PARAMETER;
    }

    // SAFETY: Cyclone DDS passes a valid configuration string, or null.
    let trace = unsafe { config_option(config, c"TRACE") }.map(std::path::PathBuf::from);

    let mut psmx = Box::new(Psmx {
        inner: zeroed(),
        trace,
    });

    psmx.inner.ops = cyclonedds_sys::dds_psmx_ops_t {
        type_qos_supported: Some(type_qos_supported),
        // Null for interface version 1.
        create_topic: None,
        delete_topic: Some(delete_topic),
        // Null for interface version 1.
        deinit: None,
        get_node_id: Some(get_node_id),
        supported_features: Some(supported_features),
        create_topic_with_type: Some(create_topic_with_type),
        delete_psmx: Some(delete_psmx),
    };

    // SAFETY: guaranteed by the caller to be a valid pointer to write through.
    unsafe { *psmx_instance = Box::into_raw(psmx).cast() };

    RETCODE_OK
}
