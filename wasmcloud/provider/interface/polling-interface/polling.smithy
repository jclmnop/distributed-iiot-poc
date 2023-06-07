// polling.smithy

// Tell the code generator how to reference symbols defined in this namespace
metadata package = [{
    namespace: "org.wasmcloud.interface.polling",
    crate: "wasmcloud_interface_polling"
}]

namespace org.wasmcloud.interface.polling

use org.wasmcloud.model#wasmbus
use org.wasmcloud.model#U32
use org.wasmcloud.model#U64

/// The Polling interface describes a service that automatically polls external
/// services/hardware at a set interval, or a service which simply ensures an
/// actor performs certain functionality at a set interval without any external
/// input from the Polling service itself.
///
/// The AddPollTarget and RemovePollTarget operations may not be necessary for
/// all implementations; some implementations might monitor heartbeats from sources
/// configured in the link definition rather than manually adding or removing
/// targets.
@wasmbus(
    contractId: "wasmcloud:polling",
    providerReceive: true )
service Polling {
    version: "0.1",
    operations: [ PollTx, AddPollTarget, RemovePollTarget ]
}

/// The PollSubscriber interface described an actor interface that receives the
/// results from automatic polling of external services/hardware at a specified
/// interval.
@wasmbus(
    contractId: "wasmcloud:polling",
    actorReceive: true )
service PollSubscriber {
    version: "0.1",
    operations: [ PollRx ]
}

/// Receive results from the latest automatic poll, or simply perform some
/// functionality without any external input
operation PollRx {
    input: PollResult,
}

/// Manually request the provider to poll any external services/hardware
operation PollTx {
    input: PollRequest,
    output: PollResult,
}

/// Add a new target to be regularly polled
operation AddPollTarget {
    input: AddPollTargetRequest,
    output: AddPollTargetResponse,
}

/// Remove a target that's currently being polled so it will no longer be polled
operation RemovePollTarget {
    input: RemovePollTargetRequest,
    output: RemovePollTargetResponse,
}

/// Results from either automatic or manual polling of external services/hardware
structure PollResult {
    /// Bytes serialised from a data structure suitable for the type of services
    /// being polled.
    data: Blob,
    /// If this field is present then there has been an error
    error: PollingError
}

/// A request to manually poll external services/hardware.
///
/// Depending on the provider implementation, leaving `Data` empty may poll all
/// services/hardware, it may perform an entirely different action, or it could
/// return an error.
structure PollRequest {
    /// Bytes which can be serialised from any type of data structure, depending
    /// on the implementation of the provider. For an HTTP based provider it could
    /// be a hash map of API endpoints and parameters, for a NATS based provider
    /// it could be a list of topics to publish to, or it could simply be a list
    /// of Sensor IDs to poll for readings.
    ///
    /// The effect of leaving this empty will depend on the provider implementation.
    request_data: Blob
}

structure AddPollTargetRequest {
    @required
    targetData: Blob,
    pollInterval: U32,
}

/// If the request was not successful it will contain an error
structure AddPollTargetResponse {
    error: PollingError
}

structure RemovePollTargetRequest {
    @required
    targetData: Blob
}

/// If the request was not successful it will contain an error
structure RemovePollTargetResponse {
    error: PollingError
}

/// Contains the type or code for an error along with an optional description.
structure PollingError {
    @required
    errorType: String,
    description: String
}

