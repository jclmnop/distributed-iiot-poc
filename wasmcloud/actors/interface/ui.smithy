// ui.smithy
//
// Heavily inspired by https://github.com/wasmCloud/examples/blob/main/petclinic/petclinic-interface/ui.smithy

// Tell the code generator how to reference symbols defined in this namespace
metadata package = [ { namespace: "jclmnop.iiot_poc.interface.ui", crate: "actor_interfaces" } ]

namespace jclmnop.iiot_poc.interface.ui

use org.wasmcloud.model#wasmbus

@wasmbus( actorReceive: true )
service Ui {
    version: "0.1",
    operations: [ GetAsset ]
}

/// Gets the asset with the given path, for example `/index.html`.
operation GetAsset {
    input: String,
    output: GetAssetResponse
}

structure GetAssetResponse {
    /// True if request was successful and asset was found, false if request was successful but asset was not found.
    @required
    found: Boolean,

    /// Optional hint to the caller about the content type of the asset. Should be a valid MIME type.
    contentType: String,

    /// Raw asset as bytes.
    @required
    asset: Blob,
}
