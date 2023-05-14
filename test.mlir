module attributes {llvm.data_layout = ""} {
  llvm.func @add(%arg0: f32, %arg1: f32) -> f32 {
    %0 = llvm.fadd %arg0, %arg1  : f32
    llvm.return %0 : f32
  }
  llvm.func @firstInVector(%arg0: vector<2xf32>) -> f32 {
    %0 = llvm.mlir.constant(0 : i64) : i64
    %1 = llvm.extractelement %arg0[%0 : i64] : vector<2xf32>
    llvm.return %1 : f32
  }
}
