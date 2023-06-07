// common.smithy
//

// Tell the code generator how to reference symbols defined in this namespace
metadata package = [ { namespace: "jclmnop.iiot_poc.interface.common", crate: "actor_interfaces" } ]

namespace jclmnop.iiot_poc.interface.common

use org.wasmcloud.model#wasmbus

/// Description of Common service
@wasmbus( actorReceive: true )
service Common {
  version: "0.1",
  operations: [ Convert ]
}

/// Converts the input string to a result
operation Convert {
  input: String,
  output: String
}

