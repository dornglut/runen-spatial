use godot::prelude::*;

mod bridge;
mod world_streaming_node;

struct GodotWorldStreamingExtension;

#[gdextension]
unsafe impl ExtensionLibrary for GodotWorldStreamingExtension {}
