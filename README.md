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

To test lowering MLIR to LLVM IR,

```bash
cd test/mlir/
mlir-translate -mlir-to-llvmir ./test_llvm.mlir >> test.ll
```

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

### Apple Support

There seems to be no MLIR dialects that support Metal or the Apple Neural Engine. There should probably be some API for Core ML.

## Pony Notes

Notes on the Pony programming language's implementation.

To test lowering Pony to LLVM IR,

```bash
cd test/emit-test-pony/
ponyc --pass ir
```

### Semantics

> The default for any mutable reference capability is `iso` and the default for any immutable reference capability is `val`.

### `libponyrt`

`libponyrt` is the runtime library for Pony. `runtime/` uses Zig to build the C source.

types:

- `pony_ctx_t` - context type
- [`pony_type_t`](https://github.com/ponylang/ponyc/blob/d82fb4a1fc1486bb7f4286f8e1195c1993eaaa34/src/libponyrt/pony.h#L153) - runtime Pony type info
  - size, field count, `void* instance`, `void* vtable`, lots of custom functions
- [`pony_actor_t`](https://github.com/ponylang/ponyc/blob/d82fb4a1fc1486bb7f4286f8e1195c1993eaaa34/src/libponyrt/actor/actor.h#L53) - main actor instance
- `pony_msg_t` - message type

`pony.h` contains:

- `pony_alloc_msg(_size)` - allocates a pony message
- `pony_send(v|i|p)(ctx, actor, msg)` - send a message to an actor
- `pony_init` - initialize the runtime
- `pony_start` - start the runtime

`actor/actor.c` contains:

- [`pony_actor_t* pony_create(pony_ctx_t* ctx, pony_type_t* type, bool orphaned)`](https://github.com/ponylang/ponyc/blob/d82fb4a1fc1486bb7f4286f8e1195c1993eaaa34/src/libponyrt/actor/actor.c#L884)
  - Creates a `pony_actor_t` given a `pony_type_t`

### `libponyc`

`libponyc` is the compiler for Pony.

This structure exists in [`codegen/gentype.h`](https://github.com/ponylang/ponyc/blob/d82fb4a1fc1486bb7f4286f8e1195c1993eaaa34/src/libponyc/codegen/gentype.h#L12):

```c
typedef struct compile_type_t
{
  compile_opaque_free_fn free_fn;

  size_t abi_size;

  LLVMTypeRef structure;
  LLVMTypeRef structure_ptr;
  LLVMTypeRef primitive;
  LLVMTypeRef use_type;
  LLVMTypeRef mem_type;

  LLVMTypeRef desc_type;
  LLVMValueRef desc;
  LLVMValueRef instance;
  LLVMValueRef trace_fn;
  LLVMValueRef serialise_trace_fn;
  LLVMValueRef serialise_fn;
  LLVMValueRef deserialise_fn;
  LLVMValueRef custom_serialise_space_fn;
  LLVMValueRef custom_serialise_fn;
  LLVMValueRef custom_deserialise_fn;
  LLVMValueRef final_fn;
  LLVMValueRef dispatch_fn;
  LLVMValueRef dispatch_switch;

  LLVMMetadataRef di_file;
  LLVMMetadataRef di_type;
  LLVMMetadataRef di_type_embed;
} compile_type_t;
```

It looks like the LLVM values here are created when making an actor type.
`desc` is passed to `pony_create` [at runtime](https://github.com/ponylang/ponyc/blob/d82fb4a1fc1486bb7f4286f8e1195c1993eaaa34/src/libponyc/codegen/genexe.c#L28-L30).

```c
// Create the main actor and become it.
LLVMValueRef args[3];
args[0] = ctx;
args[1] = ((compile_type_t*)t->c_type)->desc;
args[2] = LLVMConstInt(c->i1, 0, false);
LLVMValueRef actor = gencall_runtime(c, "pony_create", args, 3, "");
```

In `test/emit-test-pony/main.pony`, the `Main` actor is turned into this type:

```llvm
%4 = type { i32, i32, i32, i32, i1, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, ptr, i32, ptr, ptr, [2 x ptr] }

; Main Actor
@4 = private constant %4 {
  i32 1,
  i32 272,
  i32 0,
  i32 0,
  i1 true,
  ptr null,
  ptr null,
  ptr null,
  ptr null,
  ptr null, 
  ptr null, 
  ptr null, 
  ptr @Main_Dispatch, 
  ptr null, 
  i32 -1, 
  ptr @31,
  ptr null, 
  [2 x ptr] [ptr @Main_runtime_override_defaults_oo, ptr @41]
}

; in @main
%6 = tail call ptr @pony_ctx()
%7 = tail call ptr @pony_create(ptr %6, ptr nonnull @4, i1 false)
tail call void @ponyint_become(ptr %6, ptr %7)
```

No clue what that massive type contains! Looks like every actor needs a unique one.

Main function [flow](https://github.com/ponylang/ponyc/blob/d82fb4a1fc1486bb7f4286f8e1195c1993eaaa34/src/libponyc/codegen/genexe.c#L92):

- call `pony_init`
- `let ctx = pony_ctx()`
- `new Main()`
- _"Create an Env on the main actor's heap."_
  - what??
- _"Run primitive initialisers using the main actor's heap."_
  - uh I don't think I need this
- build a message `pony_alloc_msg`
  - what is this message??
- GC the message
  - `pony_gc_send` starts the sending procedure
  - `pony_traceknown` traces an object
    - called for every pointer field in an object
  - `pony_send_done` finishes gc tracing for sending
- send the message `pony_sendv_single`
- start the runtime `pony_start(boolean library, int* exit_code, bigstruct lang_features)`
  - wut how are we sending a message when the runtime hasn't started? is it just sitting in the actor queue?
