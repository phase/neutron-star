; ModuleID = 'LLVMDialectModule'
source_filename = "LLVMDialectModule"

declare ptr @malloc(i64)

declare void @free(ptr)

define float @add(float %0, float %1) {
  %3 = fadd float %0, %1
  ret float %3
}

define float @firstInVector(<2 x float> %0) {
  %2 = extractelement <2 x float> %0, i64 0
  ret float %2
}

!llvm.module.flags = !{!0}

!0 = !{i32 2, !"Debug Info Version", i32 3}
