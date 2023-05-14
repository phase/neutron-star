# Neutron Star

Programming langauge using [MLIR](https://mlir.llvm.org/).

## This looks like a plan

- [melior](https://github.com/raviqqe/melior) for high level Rust usage
  - unfinished - will need to call out to the MLIR C++ API for things like converting MLIR to LLVM IR
- make mlir dialect for pony actors & orca refs
  - lowers to llvm ir calls to the runtime C/C++ functions
- runtime in Zig that pulls in libponyrt
- source language for structs, actors, functions, gpu kernels
  - gpu kernels use gpu dialect & `gpu_to_llvm` conversion

- ml framework
  - build matrix / tensor library
    - reference counted structs? need to test performance of different memory management strategies
    - userland mlir types/ops such as tensor & linalg
    - core ml support?
  - ml primitives

## Components

- `src/` - Rust crate: compiler using MLIR
- `runtime/` - Zig exe: runtime using `libponyrt`

Some software required forking:

- [melior](https://github.com/phase/melior/tree/pub-everywhere)
- [ponyc](https://github.com/phase/ponyc/tree/compile-with-zig)

## MLIR notes

Notes on the dialects and conversions.

|name|target|
|--|--|
|`vector`|`llvm`, `gpu`, `spirv`|
|`tensor`|`linalg` (partial), `spirv`|
|`gpu`|`nvvm`, `rocdl`, `spirv`|
|`linalg`|`llvm`, `std`|
|`math`|`llvm`, `spirv`|
|`gpu-launch` | `vulkan-launch`

High level paths:

- `tensor` & `vector` can lower to `llvm` + `spirv`

### `vector` Dialect

[MLIR Docs](https://mlir.llvm.org/docs/Dialects/Vector/)

`vector` lowers to `llvm`, `gpu`, `spirv`

[MLIR Vector Dialect and Patterns](https://www.lei.chat/posts/mlir-vector-dialect-and-patterns/) by Lei Zhang.

### `tensor` Dialect

[MLIR Docs](https://mlir.llvm.org/docs/Dialects/TensorOps/)

`tensor` lowers to: `linalg`, `spirv`

`tensor` type is in the builtin dialect and is for dense multi-dimensional arrays.

### `gpu` Dialect

[MLIR Docs](https://mlir.llvm.org/docs/Dialects/GPU/)

`gpu` lowers to `nvvm`, `rocdl`, `spirv`.

`gpu.modle` is the top level compilation unit. A host device can launch `gpu.func`s using `gpu.launch_func`.

`gpu.func` is either:

- kernel that's launched from the host side
- function that is device side only

Example:

```mlir
gpu.func @foo(%arg0: index)
    workgroup(%workgroup: memref<32xf32, 3>)
    private(%private: memref<1xf32, 5>)
    kernel
    attributes {qux: "quux"} {
  gpu.return
}
```

`gpu.launc` launches a kernel function on a grid of thread blocks.

### `linalg` Dialect

[MLIR Docs](https://mlir.llvm.org/docs/Dialects/Linalg/)

`linalg` lowers to `llvm`, `std`.

Generic operators like pointwise, matmul, conv.

## Apple Support

There seems to be no MLIR dialects that support Metal or the Apple Neural Engine. There should probably be some API for Core ML.
