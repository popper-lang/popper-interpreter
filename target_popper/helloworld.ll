; ModuleID = 'examples/helloworld.pop'
source_filename = "examples/helloworld.pop"

declare i32 @print([2 x i8])

define ptr @main(i32 %0) {
entry:
  %string_literal = alloca [16 x i8], align 1
  store [16 x i8] c"Hello, world!\\n\00", ptr %string_literal, align 1
  %call_print_ = call i32 @print(ptr %string_literal)
  %return = alloca ptr, align 8
  store i32 0, ptr %return, align 4
  ret ptr %return
}
