// common.smithy
//

// Tell the code generator how to reference symbols defined in this namespace
metadata package = [ { namespace: "jclmnop.iiot_poc.interface.pangea_api", crate: "actor_interfaces" } ]

namespace jclmnop.iiot_poc.interface.pangea_api

use org.wasmcloud.model#wasmbus
use org.wasmcloud.model#U64

@wasmbus( actorReceive: true )
service PangeaApi {
    version: "0.1",
    operations: [ WriteAuditLog, SearchAuditLog ]
}

operation WriteAuditLog {
    input: LogEvents,
    output: WriteResult
}

operation SearchAuditLog {
    input: String, // TODO: struct for search params
    output: SearchResponse
}

structure WriteResult {
    @required
    success: Boolean,
    /// Only needs to be checked if success is false
    reason: String,
}

list LogEvents {
    member: LogEvent
}

/// A log event which conforms to the [Pangea API](https://pangea.cloud/docs/audit/using-secure-audit-log/log-events)
structure LogEvent {
    /// This field is used to record a detailed account of what action occurred. This can be recorded as free-form text
    /// or as a JSON field. If JSON is provided the log viewer will render this field as JSON.
    @required
    message: String,
    /// The actor field is used to record who performed a specific action. This could be used to record the user ID,
    /// username, first and last name, or a combination of fields.
    actor: String,
    /// The source field is for recording from where an activity occurred. This could be used to record a client's IP address,
    /// country of origin, the application used, etc.
    source: String,
    /// This is used to record the action that occurred. Typical values seen in this field are "Create/Read/Update/Delete,"
    /// but could also include actions specific to your application.
    action: String,
    /// Status is used to record whether or not the action was successful.
    status: String,
    /// This is used to record the specific record that was targeted by the recorded action. This could be an object ID,
    /// a username, or other identifying information.
    target: String,
    /// This is usually used in combination with "new-value." Old-value is used to record the value(s) of a record before
    /// any change made by the recorded action. If JSON is provided, the log viewer will render this field as JSON.
    old: String,
    /// Used in combination with "old," new is used to record the value(s) of a record after a change has been made by
    /// the recorded action. If JSON is provided, the log viewer will render this field as JSON.
    new: String,
    /// A Pangea-generated timestamp will always be provided with every log entry.
    /// This field is an optional client-supplied timestamp.
    timestamp: String,
    /// An optional client-supplied tenant_id
    tenant_id: String,
}

/// API response
structure SearchResponse {
    @required
    result: SearchResult,
    @required
    request_id: String,
    @required
    request_time: String,
    @required
    response_time: String,
    @required
    status: String,
    summary: String,
}

structure SearchResult {
    @required
    count: U64,
    @required
    events: Envelopes
}

list SearchErrors {
    member: SearchError
}

structure SearchError {
    @required
    error: String,
    @required
    field: String,
    @required
    value: String
}

list Envelopes {
    member: Envelope
}

structure Envelope {
    @required
    event: LogEvent,
    err: SearchErrors,
    @required
    received_at: String,
    signature: String,
    public_key: String,
    membership_proof: String,
    hash: String
}


