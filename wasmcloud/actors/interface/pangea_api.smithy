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
    input: SearchParams,
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

/// A log event which conforms to the [Pangea API](https://pangea.cloud/docs/api/audit/?focus=audit#log-an-entry)
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

/// Parameters for searching the audit log, all fields are optional
/// Conforms to the [Pangea API](https://pangea.cloud/docs/api/audit/?focus=audit#search-the-log)
structure SearchParams {
    /// A period of time or an absolute timestamp
    start: String,
    /// A period of time or an absolute timestamp
    end: String,
    /// Number of audit records to include from the first page of the results
    limit: U64,
    /// Maximum number of results to return.
    ///     min: 1
    ///     max: 10,000
    max_results: U64,
    /// Specify the sort order of the response.
    ///     Options: `asc`, `desc`
    order: String, // TODO: enum?
    /// Name of the colum to sort the results by.
    ///    Options: `actor`, `action`, `message`, `new`, `old`, `source`,
    ///             `status`, `target`, `timestamp`, `tenant_id`, `received_at`
    order_by: String, // TODO: enum?
    /// Natural search string; a space-seprated list of case-sensitive values. Enclose strings in double-quotes " to
    /// include spaces. Optionally prefix with a field ID and a colon : to limit to a specific field.
    /// e.g. `"actor:John Smith" action:Create` will search for the string "John Smith" in the actor field and the
    /// string "Create" in the action field, returning results such as `actor:John Smith Jr The Third`.
    query: String,
    /// Optional parameters to restrict the scope of the search based on specific values.
    /// e.g. `"actor": ["John Smith", "Jane Doe"]` will only return results where the actor field is either "John Smith"
    /// or "Jane Doe". "John Smith Jr The Third" will not be returned.
    search_restriction: SearchRestrictionParams,
    /// If true, include the root hash of the tree and the membership proof for each record in the response.
    ///     default: `false` (pangea API's default is true but we override this)
    verbose: Boolean,
}

list Strings {
    member: String
}

/// Parameters for restricting search results, for example if you only wanted
/// to see events from a specific actor or source
structure SearchRestrictionParams {
    /// A list of actions to restrict the search to
    action: Strings,
    /// A list of actors to restrict the search to
    actor: Strings,
    /// A list of messages to restrict the search to
    message: Strings,
    /// A list of new values to restrict the search to
    new: Strings,
    /// A list of old values to restrict the search to
    old: Strings,
    /// A list of sources to restrict the search to
    source: Strings,
    /// A list of statuses to restrict the search to
    status: Strings,
    /// A list of targets to restrict the search to
    target: Strings,
    /// A list of timestamps to restrict the search to. This is the timestamp
    /// provided by the client when logging the event.
    timestamp: Strings,
    /// A list of tenant_ids to restrict the search to
    tenant_id: Strings,
    /// A list of received_at timestamps to restrict the search to. This is the
    /// timestamp provided by Pangea when logging the event.
    received_at: Strings,
}

/// API response
structure SearchResponse {
    result: SearchResult,
    @required
    request_id: String,
    @required
    request_time: String,
    @required
    response_time: String,
    @required
    status: String,
    @required
    summary: String,
}

structure SearchResult {
    @required
    count: U64,
    @required
    events: Envelopes
    root: Root,
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
    envelope: EnvelopeInner,
    @required
    hash: String,
    @box
    published: Boolean,
    leaf_index: U64,
    membership_proof: String,
}

structure EnvelopeInner {
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

structure Root {
    @required
    url: String,
    @required
    published_at: String,
    @required
    size: U64,
    @required
    root_hash: String,
    @required
    tree_name: String,
    @required
    consistency_proof: Strings,
}


