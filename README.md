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
  - ml primitives

## MLIR notes

Notes on the dialects and conversions.

### `tensor` Dialect

[MLIR Docs](https://mlir.llvm.org/docs/Dialects/TensorOps/)

`tensor` type is in the builtin dialect and is for dense multi-dimensional arrays.

`--convert-tensor-to-` targets: `linalg`, `spirv`

#### `gpu` Dialect

[MLIR Docs](https://mlir.llvm.org/docs/Dialects/GPU/)

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
