; ModuleID = 'autarky_module'
source_filename = "autarky_module"

declare ptr @malloc(i64)

define i64 @autarky_main() {
entry:
  %calltmp = tail call i64 @lambda_0(i64 4)
  ret i64 %calltmp
}

define i64 @lambda_0(i64 %SYS_COMPUTE) {
entry:
  %calltmp = tail call i64 @lambda_1(i64 2)
  ret i64 %calltmp
}

define i64 @lambda_1(i64 %SYS_CREDITS) {
entry:
  %multmp = mul i64 %SYS_COMPUTE, 10
  %addtmp = add i64 %multmp, %SYS_CREDITS
  ret i64 %addtmp
}
