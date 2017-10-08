; Function Attrs: noinline nounwind optnone ssp uwtable
define i32 @be(i32) #0 {
  %2 = alloca i32, align 4
  store i32 %0, i32* %2, align 4
  %3 = load i32, i32* %2, align 4
  %4 = and i32 %3, 255
  %5 = shl i32 %4, 24
  %6 = load i32, i32* %2, align 4
  %7 = ashr i32 %6, 8
  %8 = and i32 %7, 255
  %9 = shl i32 %8, 16
  %10 = or i32 %5, %9
  %11 = load i32, i32* %2, align 4
  %12 = ashr i32 %11, 16
  %13 = and i32 %12, 255
  %14 = shl i32 %13, 8
  %15 = or i32 %10, %14
  %16 = load i32, i32* %2, align 4
  %17 = ashr i32 %16, 24
  %18 = and i32 %17, 255
  %19 = or i32 %15, %18
  ret i32 %19
}

; Function Attrs: noinline nounwind optnone ssp uwtable
define i32 @ldw(i8*, i32) #0 {
  %3 = alloca i8*, align 8
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store i8* %0, i8** %3, align 8
  store i32 %1, i32* %4, align 4
  %6 = load i8*, i8** %3, align 8
  %7 = load i32, i32* %4, align 4
  %8 = sext i32 %7 to i64
  %9 = getelementptr inbounds i8, i8* %6, i64 %8
  %10 = bitcast i8* %9 to i32*
  %11 = load i32, i32* %10, align 4
  store i32 %11, i32* %5, align 4
  %12 = load i32, i32* %5, align 4
  %13 = call i32 @be(i32 %12)
  ret i32 %13
}

; Function Attrs: noinline nounwind optnone ssp uwtable
define i32 @ldh(i8*, i32) #0 {
  %3 = alloca i8*, align 8
  %4 = alloca i32, align 4
  %5 = alloca i32, align 4
  store i8* %0, i8** %3, align 8
  store i32 %1, i32* %4, align 4
  %6 = load i8*, i8** %3, align 8
  %7 = load i32, i32* %4, align 4
  %8 = sext i32 %7 to i64
  %9 = getelementptr inbounds i8, i8* %6, i64 %8
  %10 = load i8, i8* %9, align 1
  %11 = zext i8 %10 to i32
  %12 = shl i32 %11, 8
  %13 = load i8*, i8** %3, align 8
  %14 = load i32, i32* %4, align 4
  %15 = sext i32 %14 to i64
  %16 = getelementptr inbounds i8, i8* %13, i64 %15
  %17 = getelementptr inbounds i8, i8* %16, i64 1
  %18 = load i8, i8* %17, align 1
  %19 = zext i8 %18 to i32
  %20 = or i32 %12, %19
  store i32 %20, i32* %5, align 4
  %21 = load i32, i32* %5, align 4
  ret i32 %21
}

; Function Attrs: noinline nounwind optnone ssp uwtable
define i32 @ldb(i8*, i32) #0 {
  %3 = alloca i8*, align 8
  %4 = alloca i32, align 4
  store i8* %0, i8** %3, align 8
  store i32 %1, i32* %4, align 4
  %5 = load i8*, i8** %3, align 8
  %6 = load i32, i32* %4, align 4
  %7 = sext i32 %6 to i64
  %8 = getelementptr inbounds i8, i8* %5, i64 %7
  %9 = load i8, i8* %8, align 1
  %10 = zext i8 %9 to i32
  ret i32 %10
}

; Function Attrs: noinline nounwind optnone ssp uwtable
define i32 @msh(i8*, i32) #0 {
  %3 = alloca i8*, align 8
  %4 = alloca i32, align 4
  store i8* %0, i8** %3, align 8
  store i32 %1, i32* %4, align 4
  %5 = load i8*, i8** %3, align 8
  %6 = load i32, i32* %4, align 4
  %7 = call i32 @ldb(i8* %5, i32 %6)
  %8 = and i32 %7, 15
  %9 = shl i32 %8, 2
  ret i32 %9
}
