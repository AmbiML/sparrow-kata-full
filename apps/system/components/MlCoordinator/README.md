# Crate descriptions

## Uses kata-os-common, not unit testable

kata-ml-interface: Outside interface used by external clients
kata-ml-component: Camkes interface, locks MLCoord
kata-ml-coordinator: Main point of logic

## Unit testable

kata-ml-shared: Shared structs used in most other crates
kata-ml-support: Unit testable code

## HAL

kata-vec-core: The HAL for the Vector Core
fake-vec-core: A stubbed out version of kata-vec-core
