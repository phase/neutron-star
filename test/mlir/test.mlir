module {
  func.func @add(%arg0: f32, %arg1: f32) -> f32 {
    %0 = arith.addf %arg0, %arg1 : f32
    return %0 : f32
  }
  func.func @firstInVector(%arg0: vector<2xf32>) -> f32 {
    %0 = vector.extract %arg0[0] : vector<2xf32>
    return %0 : f32
  }
}
