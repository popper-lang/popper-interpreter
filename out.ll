; ModuleID = './examples/helloworld.pop'
source_filename = "./examples/helloworld.pop"

@g0 = private unnamed_addr constant [3 x i8] c"%d\00", align 1

declare i32 @printf(ptr, ...)

define i32 @main() {
entry:
  %0 = alloca { i32, i32 }, align 8
  %1 = getelementptr { i32, i32 }, ptr %0, i32 0, i32 0
  store i32 1, ptr %1, align 4
  %2 = getelementptr { i32, i32 }, ptr %0, i32 0, i32 1
  store i32 2, ptr %2, align 4
  %3 = getelementptr { i32, i32 }, ptr %0, i32 0
  %4 = load i32, ptr %3, align 4
  %5 = call i32 (ptr, ...) @printf(ptr @g0, i32 %4)
  %6 = alloca i32, align 4
  store i32 %5, ptr %6, align 4
  ret i32 0
}
