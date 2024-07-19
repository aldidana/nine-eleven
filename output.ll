; ModuleID = 'nine_eleven'
source_filename = "nine_eleven"

@format_str = private unnamed_addr constant [4 x i8] c"%d\0A\00", align 1

define i1 @main(double %0, double %1) {
entry:
  %compare = fcmp ogt double %0, %1
  %compare_i32 = zext i1 %compare to i32
  %printf_call = call i32 (ptr, ...) @printf(ptr @format_str, i32 %compare_i32)
  ret i1 %compare
}

declare i32 @printf(ptr, ...)
