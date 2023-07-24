; ModuleID = 'probe6.cf69fdc057ec3c69-cgu.0'
source_filename = "probe6.cf69fdc057ec3c69-cgu.0"
target datalayout = "e-m:e-p:32:32-Fi8-i64:64-v128:64:128-a:0:32-n32-S64"
target triple = "thumbv6m-none-unknown-eabi"

; core::f64::<impl f64>::is_subnormal
; Function Attrs: inlinehint nounwind
define internal zeroext i1 @"_ZN4core3f6421_$LT$impl$u20$f64$GT$12is_subnormal17he7869ef9e650c754E"(double %self) unnamed_addr #0 {
start:
  %_2 = alloca i8, align 1
; call core::f64::<impl f64>::classify
  %0 = call i8 @"_ZN4core3f6421_$LT$impl$u20$f64$GT$8classify17h5104874784a345eeE"(double %self) #2, !range !0
  store i8 %0, ptr %_2, align 1
  %1 = load i8, ptr %_2, align 1, !range !0, !noundef !1
  %_3 = zext i8 %1 to i32
  %2 = icmp eq i32 %_3, 3
  ret i1 %2
}

; probe6::probe
; Function Attrs: nounwind
define dso_local void @_ZN6probe65probe17h361a423df2321c90E() unnamed_addr #1 {
start:
; call core::f64::<impl f64>::is_subnormal
  %_1 = call zeroext i1 @"_ZN4core3f6421_$LT$impl$u20$f64$GT$12is_subnormal17he7869ef9e650c754E"(double 1.000000e+00) #2
  ret void
}

; core::f64::<impl f64>::classify
; Function Attrs: nounwind
declare dso_local i8 @"_ZN4core3f6421_$LT$impl$u20$f64$GT$8classify17h5104874784a345eeE"(double) unnamed_addr #1

attributes #0 = { inlinehint nounwind "frame-pointer"="all" "target-cpu"="generic" "target-features"="+strict-align,+atomics-32" }
attributes #1 = { nounwind "frame-pointer"="all" "target-cpu"="generic" "target-features"="+strict-align,+atomics-32" }
attributes #2 = { nounwind }

!0 = !{i8 0, i8 5}
!1 = !{}
