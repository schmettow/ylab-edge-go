; ModuleID = 'probe8.6e00ad78473271e0-cgu.0'
source_filename = "probe8.6e00ad78473271e0-cgu.0"
target datalayout = "e-m:e-p:32:32-Fi8-i64:64-v128:64:128-a:0:32-n32-S64"
target triple = "thumbv6m-none-unknown-eabi"

; core::f64::<impl f64>::to_ne_bytes
; Function Attrs: inlinehint nounwind
define internal void @"_ZN4core3f6421_$LT$impl$u20$f64$GT$11to_ne_bytes17ha29542b5661a781fE"(ptr sret([8 x i8]) %0, double %self) unnamed_addr #0 {
start:
  %_3 = alloca double, align 8
  store double %self, ptr %_3, align 8
  %rt = load double, ptr %_3, align 8, !noundef !0
  %self1 = bitcast double %rt to i64
  store i64 %self1, ptr %0, align 1
  ret void
}

; probe8::probe
; Function Attrs: nounwind
define dso_local void @_ZN6probe85probe17h118e3826848c50a4E() unnamed_addr #1 {
start:
  %_1 = alloca [8 x i8], align 1
; call core::f64::<impl f64>::to_ne_bytes
  call void @"_ZN4core3f6421_$LT$impl$u20$f64$GT$11to_ne_bytes17ha29542b5661a781fE"(ptr sret([8 x i8]) %_1, double 3.140000e+00) #2
  ret void
}

attributes #0 = { inlinehint nounwind "frame-pointer"="all" "target-cpu"="generic" "target-features"="+strict-align,+atomics-32" }
attributes #1 = { nounwind "frame-pointer"="all" "target-cpu"="generic" "target-features"="+strict-align,+atomics-32" }
attributes #2 = { nounwind }

!0 = !{}
