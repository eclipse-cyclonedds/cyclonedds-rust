//! This holds the state masks that provide information on the state of a
//! sample, of a view, or of an instance.

bitflags::bitflags! {
    /// Flags that represent the set of states a sample, view, or instance can
    /// be in. These are represented privately as a single bit-mask but these
    /// fields are then re-exported in the [`sample`], [`view`], and
    /// [`instance`] modules for better organization.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct State: u32 {
        #[doc(hidden)]
        const SampleStale = cyclonedds_sys::DDS_READ_SAMPLE_STATE;
        #[doc(hidden)]
        const SampleFresh = cyclonedds_sys::DDS_NOT_READ_SAMPLE_STATE;
        #[doc(hidden)]
        const SampleAny = cyclonedds_sys::DDS_ANY_SAMPLE_STATE;
        #[doc(hidden)]
        const ViewNew = cyclonedds_sys::DDS_NEW_VIEW_STATE;
        #[doc(hidden)]
        const ViewOld = cyclonedds_sys::DDS_NOT_NEW_VIEW_STATE;
        #[doc(hidden)]
        const ViewAny = cyclonedds_sys::DDS_ANY_VIEW_STATE;
        #[doc(hidden)]
        const InstanceAlive = cyclonedds_sys::DDS_ALIVE_INSTANCE_STATE;
        #[doc(hidden)]
        const InstanceDisposed = cyclonedds_sys::DDS_NOT_ALIVE_DISPOSED_INSTANCE_STATE;
        #[doc(hidden)]
        const InstanceUnregistered = cyclonedds_sys::DDS_NOT_ALIVE_NO_WRITERS_INSTANCE_STATE;
        #[doc(hidden)]
        const InstanceAny = cyclonedds_sys::DDS_ANY_INSTANCE_STATE;
    }
}

pub mod sample {
    //! This module holds the sample state masks that provide information on the
    //! state of a sample in the sample info from a read or take call.

    #![allow(non_upper_case_globals)]
    use super::State;

    /// The sample is stale (has already been read by the reader).
    pub const Stale: State = State::SampleStale;
    /// The sample is fresh (in an unread state).
    pub const Fresh: State = State::SampleFresh;
    /// The sample is in any read state.
    pub const Any: State = State::SampleAny;
}

pub mod view {
    //! This module holds the set of states of the view of an instance relative
    //! to the samples.

    #![allow(non_upper_case_globals)]
    use super::State;

    /// The view is new (the sample is being accessed by the reader for the
    /// first time when the instance is alive).
    pub const New: State = State::ViewNew;
    /// The view is old (the reader has accessed the sample before).
    pub const Old: State = State::ViewOld;
    /// The view is in any view state.
    pub const Any: State = State::ViewAny;
}

pub mod instance {
    //! This module holds the set of states of an instance.

    #![allow(non_upper_case_globals)]
    use super::State;

    /// The instance is alive (is not disposed and has active writers).
    pub const Alive: State = State::InstanceAlive;
    /// The instance is disposed (was explicitly disposed by the writer).
    pub const Disposed: State = State::InstanceDisposed;
    /// The instance is unregistered (has been declared as not alive by the
    /// reader as there are no live writers writing that instance).
    pub const Unregistered: State = State::InstanceUnregistered;
    /// The instance is in any instance state.
    pub const Any: State = State::InstanceAny;
}
