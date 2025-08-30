#ifndef WRAPPER_H_
#define WRAPPER_H_

#include <dds/dds.h>
#include <dds/ddsc/dds_public_impl.h>
#include <dds/ddsc/dds_public_qosdefs.h>
#include <dds/ddsi/ddsi_radmin.h>
#include <dds/ddsi/ddsi_serdata.h>
#include <dds/ddsi/ddsi_sertype.h>

// Wrapper Functions:
//
// Inline functions can't be processed in a sensible fashion by `bindgen`. Such
// functions that need to be accessed are wrapped here with the
// `_bindgen_wrapper` suffix. The `build.rs` strips these suffixes out when
// generating the binding and remaps the underlying symbol.  As a result, these
// functions can be accessed on Rust side without the `_bindgen_wrapper` suffix.
//
// If the function they wrap is not inline and the wrapper has the same name
// this will lead to a failure in compilation as duplicate symbols will be
// present at link time. This is desirable as the wrappers may be added safely
// for the inline functions and if they change downstream to no longer be inline
// the wrapper function may simply be removed.

/// Increments the reference count of a `ddsi_serdata` object.
static struct ddsi_serdata* ddsi_serdata_ref_bindgen_wrapper(
    const struct ddsi_serdata* serdata_const) {
  return ddsi_serdata_ref(serdata_const);
}

/// Decrements the reference count of a `ddsi_serdata` object.
static void ddsi_serdata_unref_bindgen_wrapper(struct ddsi_serdata* serdata) {
  ddsi_serdata_unref(serdata);
}

/// The value that represents the default domain ID.
static const dds_domainid_t DOMAIN_DEFAULT = DDS_DOMAIN_DEFAULT;
/// The value that represents an infinite duration.
static const dds_duration_t DURATION_INFINITE = DDS_INFINITY;
/// The value that represents a time that is not reachable.
static const dds_time_t TIME_NEVER = DDS_NEVER;
/// Pseudo topic for the DcpsParticipant builtin topic.
static const dds_entity_t BUILTIN_TOPIC_DCPS_PARTICIPANT =
    DDS_BUILTIN_TOPIC_DCPSPARTICIPANT;
/// Pseudo topic for the DcpsTopic builtin topic.
///
/// NOTE this only works when Cyclone is built with `ENABLE_TOPIC_DISCOVERY`
/// enabled.
static const dds_entity_t BUILTIN_TOPIC_DCPS_TOPIC =
    DDS_BUILTIN_TOPIC_DCPSTOPIC;
/// Pseudo topic for the DcpsPublication builtin topic.
static const dds_entity_t BUILTIN_TOPIC_DCPS_PUBLICATION =
    DDS_BUILTIN_TOPIC_DCPSPUBLICATION;
/// Pseudo topic for the DcpsSubscription builtin topic.
static const dds_entity_t BUILTIN_TOPIC_DCPS_SUBSCRIPTION =
    DDS_BUILTIN_TOPIC_DCPSSUBSCRIPTION;

#endif  // WRAPPER_H_
