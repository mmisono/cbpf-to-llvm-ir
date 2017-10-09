# cbpf-to-llvm-ir
[![Linux Build Status](https://travis-ci.org/mmisono/cbpf-to-llvm-ir.svg?branch=master)](https://travis-ci.org/mmisono/cbpf-to-llvm-ir)

Convert cBPF program to LLVM IR

## cbpf2ir
This crate has a program called `cbpf2ir`, which can generate LLVM IR from libpcap's filter expressions.

```sh
% cargo run --bin cbpf2ir -- --help
cbpf2ir 0.1.0
Convert cBPF program to LLVM IR from libpcap's expression

USAGE:
    cbpf2ir [FLAGS] [OPTIONS] <expression> --outfile <outfile>

FLAGS:
    -d, --debug      Activate debug mode
    -h, --help       Prints help information
    -n, --noopt      no optimization
    -V, --version    Prints version information

OPTIONS:
    -l, --linktype <linktype>    LinkType (http://www.tcpdump.org/linktypes.html) [default: 1]
    -o, --outfile <outfile>      Output file

ARGS:
    <expression>    cBPF filter expression
```

## Example
```sh
cargo run --bin cbpf2ir -- -o a.ll "tcp port 80 and (((ip[2:2] - ((ip[0]&0xf)<<2)) - ((tcp[12]&0xf0)>>2)) != 0)"
```

cBPF program:
```
LD H ABS   {code: 28, jt: 00, jf: 00, k: 0000000C}  ldh [12]
JEQ K      {code: 15, jt: 00, jf: 06, k: 000086DD}  jeq 34525 0 6
LD B ABS   {code: 30, jt: 00, jf: 00, k: 00000014}  ldb [20]
JEQ K      {code: 15, jt: 00, jf: 04, k: 00000006}  jeq 6 0 4
LD H ABS   {code: 28, jt: 00, jf: 00, k: 00000036}  ldh [54]
JEQ K      {code: 15, jt: 0E, jf: 00, k: 00000050}  jeq 80 14 0
LD H ABS   {code: 28, jt: 00, jf: 00, k: 00000038}  ldh [56]
JEQ K      {code: 15, jt: 0C, jf: 00, k: 00000050}  jeq 80 12 0
LD H ABS   {code: 28, jt: 00, jf: 00, k: 0000000C}  ldh [12]
JEQ K      {code: 15, jt: 00, jf: 45, k: 00000800}  jeq 2048 0 69
LD B ABS   {code: 30, jt: 00, jf: 00, k: 00000017}  ldb [23]
JEQ K      {code: 15, jt: 00, jf: 43, k: 00000006}  jeq 6 0 67
LD H ABS   {code: 28, jt: 00, jf: 00, k: 00000014}  ldh [20]
JSET K     {code: 45, jt: 41, jf: 00, k: 00001FFF}  jset 8191 65 0
LDX B MSH  {code: B1, jt: 00, jf: 00, k: 0000000E}  ldxb ([14] & 0xf) << 2
LD H IND   {code: 48, jt: 00, jf: 00, k: 0000000E}  ldh [14+X]
JEQ K      {code: 15, jt: 03, jf: 00, k: 00000050}  jeq 80 3 0
LDX B MSH  {code: B1, jt: 00, jf: 00, k: 0000000E}  ldxb ([14] & 0xf) << 2
LD H IND   {code: 48, jt: 00, jf: 00, k: 00000010}  ldh [16+X]
JEQ K      {code: 15, jt: 00, jf: 3B, k: 00000050}  jeq 80 0 59
LD H ABS   {code: 28, jt: 00, jf: 00, k: 0000000C}  ldh [12]
JEQ K      {code: 15, jt: 00, jf: 39, k: 00000800}  jeq 2048 0 57
LD IMM     {code: 00, jt: 00, jf: 00, k: 00000002}  ldw 2
ST         {code: 02, jt: 00, jf: 00, k: 00000000}  st MEM[0]
LDX MEM    {code: 61, jt: 00, jf: 00, k: 00000000}  ldxw MEM[0]
LD H IND   {code: 48, jt: 00, jf: 00, k: 0000000E}  ldh [14+X]
ST         {code: 02, jt: 00, jf: 00, k: 00000001}  st MEM[1]
LD IMM     {code: 00, jt: 00, jf: 00, k: 00000000}  ldw 0
ST         {code: 02, jt: 00, jf: 00, k: 00000002}  st MEM[2]
LDX MEM    {code: 61, jt: 00, jf: 00, k: 00000002}  ldxw MEM[2]
LD B IND   {code: 50, jt: 00, jf: 00, k: 0000000E}  ldb [14+X]
ST         {code: 02, jt: 00, jf: 00, k: 00000003}  st MEM[3]
LD IMM     {code: 00, jt: 00, jf: 00, k: 0000000F}  ldw 15
ST         {code: 02, jt: 00, jf: 00, k: 00000004}  st MEM[4]
LDX MEM    {code: 61, jt: 00, jf: 00, k: 00000004}  ldxw MEM[4]
LD MEM     {code: 60, jt: 00, jf: 00, k: 00000003}  ldw MEM[3]
AND X      {code: 5C, jt: 00, jf: 00, k: 00000000}  and X
ST         {code: 02, jt: 00, jf: 00, k: 00000004}  st MEM[4]
LD IMM     {code: 00, jt: 00, jf: 00, k: 00000002}  ldw 2
ST         {code: 02, jt: 00, jf: 00, k: 00000005}  st MEM[5]
LDX MEM    {code: 61, jt: 00, jf: 00, k: 00000005}  ldxw MEM[5]
LD MEM     {code: 60, jt: 00, jf: 00, k: 00000004}  ldw MEM[4]
LSH X      {code: 6C, jt: 00, jf: 00, k: 00000000}  lsh X
ST         {code: 02, jt: 00, jf: 00, k: 00000005}  st MEM[5]
LDX MEM    {code: 61, jt: 00, jf: 00, k: 00000005}  ldxw MEM[5]
LD MEM     {code: 60, jt: 00, jf: 00, k: 00000001}  ldw MEM[1]
SUB X      {code: 1C, jt: 00, jf: 00, k: 00000000}  sub X
ST         {code: 02, jt: 00, jf: 00, k: 00000005}  st MEM[5]
LD IMM     {code: 00, jt: 00, jf: 00, k: 0000000C}  ldw 12
ST         {code: 02, jt: 00, jf: 00, k: 00000006}  st MEM[6]
LDX B MSH  {code: B1, jt: 00, jf: 00, k: 0000000E}  ldxb ([14] & 0xf) << 2
LD MEM     {code: 60, jt: 00, jf: 00, k: 00000006}  ldw MEM[6]
ADD X      {code: 0C, jt: 00, jf: 00, k: 00000000}  add X
TAX        {code: 07, jt: 00, jf: 00, k: 00000000}  tax
LD B IND   {code: 50, jt: 00, jf: 00, k: 0000000E}  ldb [14+X]
ST         {code: 02, jt: 00, jf: 00, k: 00000007}  st MEM[7]
LD IMM     {code: 00, jt: 00, jf: 00, k: 000000F0}  ldw 240
ST         {code: 02, jt: 00, jf: 00, k: 00000008}  st MEM[8]
LDX MEM    {code: 61, jt: 00, jf: 00, k: 00000008}  ldxw MEM[8]
LD MEM     {code: 60, jt: 00, jf: 00, k: 00000007}  ldw MEM[7]
AND X      {code: 5C, jt: 00, jf: 00, k: 00000000}  and X
ST         {code: 02, jt: 00, jf: 00, k: 00000008}  st MEM[8]
LD IMM     {code: 00, jt: 00, jf: 00, k: 00000002}  ldw 2
ST         {code: 02, jt: 00, jf: 00, k: 00000009}  st MEM[9]
LDX MEM    {code: 61, jt: 00, jf: 00, k: 00000009}  ldxw MEM[9]
LD MEM     {code: 60, jt: 00, jf: 00, k: 00000008}  ldw MEM[8]
RSH X      {code: 7C, jt: 00, jf: 00, k: 00000000}  rsh X
ST         {code: 02, jt: 00, jf: 00, k: 00000009}  st MEM[9]
LDX MEM    {code: 61, jt: 00, jf: 00, k: 00000009}  ldxw MEM[9]
LD MEM     {code: 60, jt: 00, jf: 00, k: 00000005}  ldw MEM[5]
SUB X      {code: 1C, jt: 00, jf: 00, k: 00000000}  sub X
ST         {code: 02, jt: 00, jf: 00, k: 00000009}  st MEM[9]
LD IMM     {code: 00, jt: 00, jf: 00, k: 00000000}  ldw 0
ST         {code: 02, jt: 00, jf: 00, k: 0000000A}  st MEM[10]
LDX MEM    {code: 61, jt: 00, jf: 00, k: 0000000A}  ldxw MEM[10]
LD MEM     {code: 60, jt: 00, jf: 00, k: 00000009}  ldw MEM[9]
SUB X      {code: 1C, jt: 00, jf: 00, k: 00000000}  sub X
JEQ K      {code: 15, jt: 01, jf: 00, k: 00000000}  jeq 0 1 0
RET K      {code: 06, jt: 00, jf: 00, k: 0000FFFF}  ret 65535
RET K      {code: 06, jt: 00, jf: 00, k: 00000000}  ret 0
```

LLVM IR (after optimization)

```
LLVM IR:
; ModuleID = 'cbpf_ir'
source_filename = "cbpf_ir"

; Function Attrs: norecurse nounwind readonly
define i32 @main(i8* nocapture readonly) local_unnamed_addr #0 {
entry:
  %1 = getelementptr inbounds i8, i8* %0, i64 12
  %2 = load i8, i8* %1, align 1
  %3 = zext i8 %2 to i32
  %4 = shl nuw nsw i32 %3, 8
  %5 = getelementptr inbounds i8, i8* %0, i64 13
  %6 = load i8, i8* %5, align 1
  %7 = zext i8 %6 to i32
  %8 = or i32 %4, %7
  %cond = icmp eq i32 %8, 2048
  br i1 %cond, label %insn.10, label %insn.79

insn.10:                                          ; preds = %entry
  %9 = getelementptr inbounds i8, i8* %0, i64 23
  %10 = load i8, i8* %9, align 1
  %11 = icmp eq i8 %10, 6
  br i1 %11, label %insn.12, label %insn.79

insn.12:                                          ; preds = %insn.10
  %12 = getelementptr inbounds i8, i8* %0, i64 20
  %13 = load i8, i8* %12, align 1
  %14 = zext i8 %13 to i32
  %15 = shl nuw nsw i32 %14, 8
  %16 = getelementptr inbounds i8, i8* %0, i64 21
  %17 = load i8, i8* %16, align 1
  %18 = zext i8 %17 to i32
  %.masked = and i32 %15, 7936
  %19 = or i32 %.masked, %18
  %20 = icmp eq i32 %19, 0
  br i1 %20, label %insn.14, label %insn.79

insn.14:                                          ; preds = %insn.12
  %21 = getelementptr inbounds i8, i8* %0, i64 14
  %22 = load i8, i8* %21, align 1
  %23 = zext i8 %22 to i32
  %24 = shl nuw nsw i32 %23, 2
  %25 = and i32 %24, 60
  %26 = add nuw nsw i32 %25, 14
  %27 = zext i32 %26 to i64
  %28 = getelementptr inbounds i8, i8* %0, i64 %27
  %29 = load i8, i8* %28, align 1
  %30 = zext i8 %29 to i32
  %31 = shl nuw nsw i32 %30, 8
  %32 = getelementptr inbounds i8, i8* %28, i64 1
  %33 = load i8, i8* %32, align 1
  %34 = zext i8 %33 to i32
  %35 = or i32 %31, %34
  %36 = icmp eq i32 %35, 80
  br i1 %36, label %insn.22, label %insn.17

insn.17:                                          ; preds = %insn.14
  %37 = add nuw nsw i32 %25, 16
  %38 = zext i32 %37 to i64
  %39 = getelementptr inbounds i8, i8* %0, i64 %38
  %40 = load i8, i8* %39, align 1
  %41 = zext i8 %40 to i32
  %42 = shl nuw nsw i32 %41, 8
  %43 = getelementptr inbounds i8, i8* %39, i64 1
  %44 = load i8, i8* %43, align 1
  %45 = zext i8 %44 to i32
  %46 = or i32 %42, %45
  %47 = icmp eq i32 %46, 80
  br i1 %47, label %insn.22, label %insn.79

insn.22:                                          ; preds = %insn.17, %insn.14
  %48 = getelementptr inbounds i8, i8* %0, i64 16
  %49 = load i8, i8* %48, align 1
  %50 = zext i8 %49 to i32
  %51 = shl nuw nsw i32 %50, 8
  %52 = getelementptr inbounds i8, i8* %0, i64 17
  %53 = load i8, i8* %52, align 1
  %54 = zext i8 %53 to i32
  %55 = or i32 %51, %54
  %56 = sub nsw i32 %55, %25
  %57 = add nuw nsw i32 %25, 26
  %58 = zext i32 %57 to i64
  %59 = getelementptr inbounds i8, i8* %0, i64 %58
  %60 = load i8, i8* %59, align 1
  %61 = and i8 %60, -16
  %62 = zext i8 %61 to i32
  %63 = lshr exact i32 %62, 2
  %64 = icmp eq i32 %56, %63
  br i1 %64, label %insn.79, label %insn.78

insn.78:                                          ; preds = %insn.79, %insn.22
  %merge = phi i32 [ 65535, %insn.22 ], [ 0, %insn.79 ]
  ret i32 %merge

insn.79:                                          ; preds = %entry, %insn.12, %insn.22, %insn.17, %insn.10
  br label %insn.78
}

; Function Attrs: nounwind readnone
define i32 @be(i32) local_unnamed_addr #1 {
  %2 = tail call i32 @llvm.bswap.i32(i32 %0)
  ret i32 %2
}

; Function Attrs: nounwind readnone speculatable
declare i32 @llvm.bswap.i32(i32) #2

attributes #0 = { norecurse nounwind readonly }
attributes #1 = { nounwind readnone }
attributes #2 = { nounwind readnone speculatable }
```

eBPF code (`llc -march=bpf -o a.bpf a.ll`)
```asm
        .text
        .macosx_version_min 10, 12
        .globl  main                    # -- Begin function main
        .p2align        3
main:                                   # @main
# BB#0:                                 # %entry
        r2 = *(u8 *)(r1 + 13)
        r3 = *(u8 *)(r1 + 12)
        r3 <<= 8
        r3 |= r2
        if r3 != 2048 goto LBB0_7
# BB#1:                                 # %insn.10
        r2 = *(u8 *)(r1 + 23)
        if r2 != 6 goto LBB0_7
# BB#2:                                 # %insn.12
        r2 = *(u8 *)(r1 + 21)
        r3 = *(u8 *)(r1 + 20)
        r3 <<= 8
        r3 &= 7936
        r3 |= r2
        if r3 != 0 goto LBB0_7
# BB#3:                                 # %insn.14
        r2 = *(u8 *)(r1 + 14)
        r2 <<= 2
        r2 &= 60
        r3 = r1
        r3 += r2
        r4 = *(u8 *)(r3 + 15)
        r3 = *(u8 *)(r3 + 14)
        r3 <<= 8
        r3 |= r4
        if r3 == 80 goto LBB0_5
# BB#4:                                 # %insn.17
        r3 = r2
        r3 <<= 32
        r3 >>= 32
        r4 = r1
        r4 += r3
        r3 = *(u8 *)(r4 + 17)
        r4 = *(u8 *)(r4 + 16)
        r4 <<= 8
        r4 |= r3
        if r4 != 80 goto LBB0_7
LBB0_5:                                 # %insn.22
        r3 = *(u8 *)(r1 + 16)
        r3 <<= 8
        r4 = *(u8 *)(r1 + 17)
        r3 |= r4
        r3 -= r2
        r2 <<= 32
        r2 >>= 32
        r1 += r2
        r0 = 65535
        r1 = *(u8 *)(r1 + 26)
        r1 &= 240
        r1 >>= 2
        if r3 == r1 goto LBB0_7
LBB0_6:                                 # %insn.78
        exit
LBB0_7:                                 # %insn.79
        r0 = 0
        goto LBB0_6
```

## Convertion Strategy
Convert each cBPF instruction to the corresponding basic block.
Some instructions which are difficult to directly convert LLVM IR are
converted so as to call functions defined in [src/ll/util.ll](./src/ll/util.ll),
which is generated by `clang -S -emit-llvm util.c`.
All of these functions are inlined by optimization.
(Note that to compile eBPF program, all functions must be inlined)

## Note
The converted codes are not verified well yet.

## Related Project
- [c2e](https://github.com/mmisono/rust-cbpf/tree/master/c2e): Convert a cBPF program directly to the eBPF program

## License
Dual-licensed under [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE).
