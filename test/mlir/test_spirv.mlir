spirv.module Logical Vulkan
    requires #spirv.vce<v1.0, [Shader], [SPV_KHR_vulkan_memory_model]> {
  spirv.func @add(%arg0: f32, %arg1: f32) -> f32 "None" {
    %0 = spirv.FAdd %arg0, %arg1 : f32
    spirv.ReturnValue %0 : f32
  }
  spirv.func @firstInVector(%arg0: vector<2xf32>) -> f32 "None" {
    %0 = spirv.CompositeExtract %arg0[0 : i32] : vector<2xf32>
    spirv.ReturnValue %0 : f32
  }
}