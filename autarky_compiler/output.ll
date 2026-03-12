; ModuleID = 'autarky_module'
source_filename = "autarky_module"

declare ptr @malloc(i64)

define i64 @autarky_main() {
entry:
  %calltmp = tail call i64 @lambda_0(i64 42)
  ret i64 %calltmp
}

define i64 @rec_sum_n(i64 %s) {
entry:
  %pcast = inttoptr i64 %s to ptr
  %u0 = getelementptr inbounds { i64, i64 }, ptr %pcast, i32 0, i32 0
  %u1 = getelementptr inbounds { i64, i64 }, ptr %pcast, i32 0, i32 1
  %v1 = load i64, ptr %u0, align 4
  %v2 = load i64, ptr %u1, align 4
  %eqtmp = icmp eq i64 %v1, 0
  br i1 %eqtmp, label %left_branch, label %right_branch

left_branch:                                      ; preds = %entry
  br label %match_cont

right_branch:                                     ; preds = %entry
  %subtmp = sub i64 %v1, 1
  %addtmp = add i64 %v2, %v1
  %malloc_pair = call ptr @malloc(i64 ptrtoint (ptr getelementptr ({ i64, i64 }, ptr null, i32 1) to i64))
  %p0 = getelementptr inbounds { i64, i64 }, ptr %malloc_pair, i32 0, i32 0
  store i64 %subtmp, ptr %p0, align 4
  %p1 = getelementptr inbounds { i64, i64 }, ptr %malloc_pair, i32 0, i32 1
  store i64 %addtmp, ptr %p1, align 4
  %arg_cast = ptrtoint ptr %malloc_pair to i64
  %calltmp = tail call i64 @rec_sum_n(i64 %arg_cast)
  br label %match_cont

match_cont:                                       ; preds = %right_branch, %left_branch
  %matchtmp = phi i64 [ %v2, %left_branch ], [ %calltmp, %right_branch ]
  ret i64 %matchtmp
}

define i64 @lambda_0(i64 %init) {
entry:
  %malloc_pair = call ptr @malloc(i64 ptrtoint (ptr getelementptr ({ i64, i64 }, ptr null, i32 1) to i64))
  %p0 = getelementptr inbounds { i64, i64 }, ptr %malloc_pair, i32 0, i32 0
  store i64 10, ptr %p0, align 4
  %p1 = getelementptr inbounds { i64, i64 }, ptr %malloc_pair, i32 0, i32 1
  store i64 0, ptr %p1, align 4
  %arg_cast = ptrtoint ptr %malloc_pair to i64
  %calltmp = tail call i64 @rec_sum_n(i64 %arg_cast)
  ret i64 %calltmp
}
