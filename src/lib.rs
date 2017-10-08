extern crate cbpf;
extern crate libc;
extern crate llvm_sys as llvm;

use cbpf::opcode::*;
use llvm::prelude::*;
use llvm::analysis::{LLVMVerifierFailureAction, LLVMVerifyModule};
use llvm::execution_engine::{LLVMExecutionEngineRef, LLVMMCJITCompilerOptions};

use std::ptr;
use std::mem;
use std::collections::HashMap;

macro_rules! cstr {
    ($x: expr) => (concat!($x, "\0").as_ptr() as *const libc::c_char);
    () => (b"\0".as_ptr() as *const libc::c_char);
}

type Func = extern "C" fn(*mut i8) -> i32;

pub struct Converter {
    context: LLVMContextRef,
    module: LLVMModuleRef,
    builder: LLVMBuilderRef,
    functions: HashMap<String, LLVMValueRef>,
    values: HashMap<String, LLVMValueRef>,
    engine: Option<LLVMExecutionEngineRef>,
    jit_func: Option<Func>,
}

// it seems IRParse requires null terminated strings
static UTIL_CODE: &'static str = concat!(include_str!("./ll/util.ll"), "\0");

// TODO: error handling, currently just panic!() if something go wrong
impl Converter {
    pub fn new() -> Self {
        unsafe {
            llvm::target::LLVM_InitializeNativeTarget();
            llvm::target::LLVM_InitializeNativeAsmPrinter();
            llvm::target::LLVM_InitializeNativeAsmParser();

            let context = llvm::core::LLVMContextCreate();
            if context.is_null() {
                panic!("context is null");
            }

            let module = llvm::core::LLVMModuleCreateWithNameInContext(cstr!("cbpf_ir"), context);
            if module.is_null() {
                llvm::core::LLVMContextDispose(context);
                panic!("module is null");
            }

            let builder = llvm::core::LLVMCreateBuilderInContext(context);
            if builder.is_null() {
                llvm::core::LLVMContextDispose(context);
                llvm::core::LLVMDisposeModule(module);
                panic!("builder is null");
            }

            let values = HashMap::new();
            let functions = HashMap::new();
            let engine = None;
            let jit_func = None;

            Converter {
                context,
                module,
                builder,
                functions,
                values,
                engine,
                jit_func,
            }
        }
    }

    fn link_module_from_buf(&self, buf: LLVMMemoryBufferRef) {
        unsafe {
            let mut module: LLVMModuleRef = mem::uninitialized();
            let mut err_msg: *mut i8 = mem::uninitialized();
            let r =
                llvm::ir_reader::LLVMParseIRInContext(self.context, buf, &mut module, &mut err_msg);
            if r as i32 != 0 {
                panic!(std::ffi::CStr::from_ptr(err_msg).to_string_lossy());
            }
            if self.verify_module(module) {
                panic!("module error");
            }
            // link util
            llvm::linker::LLVMLinkModules2(self.module, module);
        }
    }

    fn load_function<I>(&mut self, name: I)
    where
        I: Into<String> + AsRef<str>,
    {
        let f = unsafe {
            llvm::core::LLVMGetNamedFunction(
                self.module,
                std::ffi::CString::new(name.as_ref()).unwrap().as_ptr(),
            )
        };
        self.functions.insert(name.into(), f);
    }

    fn link_util(&mut self) {
        unsafe {
            let buf = llvm::core::LLVMCreateMemoryBufferWithMemoryRange(
                UTIL_CODE.as_ptr() as *const _,
                (UTIL_CODE.len() - 1) as _, // exclude null terminator
                cstr!("util"),
                1,
            );
            self.link_module_from_buf(buf);

            self.load_function("ldw");
            self.load_function("ldh");
            self.load_function("ldb");
            self.load_function("msh");
        }
    }

    fn create_main(&mut self) {
        unsafe {
            let ty_i32 = llvm::core::LLVMInt32TypeInContext(self.context);
            let ty_i8 = llvm::core::LLVMInt8TypeInContext(self.context);
            let params = [llvm::core::LLVMPointerType(ty_i8, 0)];
            let ty_function = llvm::core::LLVMFunctionType(ty_i32, params.as_ptr() as *mut _, 1, 0);
            let function = llvm::core::LLVMAddFunction(self.module, cstr!("main"), ty_function);
            self.functions.insert("main".to_owned(), function);
        }
    }

    fn emit_prolog(&mut self) {
        unsafe {
            // init A, X, MEM[BPF_INSN]
            let ty_i32 = llvm::core::LLVMInt32TypeInContext(self.context);
            let bb = llvm::core::LLVMAppendBasicBlockInContext(
                self.context,
                self.get_function("main"),
                cstr!("entry"),
            );
            llvm::core::LLVMPositionBuilderAtEnd(self.builder, bb);
            let a = llvm::core::LLVMBuildAlloca(self.builder, ty_i32, cstr!("A"));
            let x = llvm::core::LLVMBuildAlloca(self.builder, ty_i32, cstr!("X"));
            let memsize = llvm::core::LLVMConstInt(ty_i32, cbpf::opcode::BPF_MEMWORDS as _, 0);
            let mem = llvm::core::LLVMBuildArrayAlloca(self.builder, ty_i32, memsize, cstr!("MEM"));
            let v = llvm::core::LLVMConstInt(ty_i32, 0, 1);
            llvm::core::LLVMBuildStore(self.builder, v, a);
            llvm::core::LLVMBuildStore(self.builder, v, x);
            self.values.insert("A".to_owned(), a);
            self.values.insert("X".to_owned(), x);
            self.values.insert("MEM".to_owned(), mem);
        }
    }

    fn get_function(&self, name: &str) -> LLVMValueRef {
        *self.functions.get(name).unwrap()
    }

    fn get_value(&self, name: &str) -> LLVMValueRef {
        *self.values.get(name).unwrap()
    }

    fn verify_module(&self, module: LLVMModuleRef) -> bool {
        let result = unsafe {
            LLVMVerifyModule(
                module,
                LLVMVerifierFailureAction::LLVMPrintMessageAction,
                ptr::null_mut(),
            )
        };
        (result as i32) != 0
    }

    fn verify_main(&self) -> bool {
        self.verify_module(self.module)
    }

    pub fn dump_module(&self) {
        unsafe {
            llvm::core::LLVMDumpModule(self.module);
        }
    }

    pub fn get_ir(&self) -> String {
        unsafe {
            std::ffi::CStr::from_ptr(llvm::core::LLVMPrintModuleToString(self.module))
                .to_string_lossy()
                .into_owned()
        }
    }

    // create basic block in advance
    fn create_basic_blocks(&mut self, num: usize) -> Vec<LLVMBasicBlockRef> {
        let mut bbs = vec![];
        for i in 0..num {
            let bb = unsafe {
                llvm::core::LLVMAppendBasicBlockInContext(
                    self.context,
                    self.get_function("main"),
                    format!("insn.{}\0", i).as_ptr() as *const _,
                )
            };
            bbs.push(bb);
        }
        bbs
    }

    pub fn convert(&mut self, insns: &[BpfInsn]) -> Result<String, String> {
        // setup
        self.create_main();
        self.link_util();
        self.emit_prolog();
        let bbs = self.create_basic_blocks(insns.len());

        // convert each instruction
        for (i, insn) in insns.iter().enumerate() {
            self.convert_insn(*insn, &bbs, i);
        }

        if self.verify_main() {
            return Err("Verify Failed".to_owned());
        }
        Ok(self.get_ir())
    }

    // should return Result
    fn convert_insn(&mut self, insn: BpfInsn, bbs: &Vec<LLVMBasicBlockRef>, idx: usize) {
        // we load A and X regardless of instructions, since they are basicaly used
        let (addr_a, addr_x, addr_mem, a, x, k, data, ty_i32) = unsafe {
            // create branch from the last bb if the last instruction is not a terminator instruction
            if idx >= 1 {
                let inst = llvm::core::LLVMGetLastInstruction(bbs[idx - 1]);
                if llvm::core::LLVMIsATerminatorInst(inst) as i32 == 0 {
                    llvm::core::LLVMBuildBr(self.builder, bbs[idx]);
                }
            } else {
                // br from entry
                llvm::core::LLVMBuildBr(self.builder, bbs[idx]);
            }
            llvm::core::LLVMPositionBuilderAtEnd(self.builder, bbs[idx]);
            let ty_i32 = llvm::core::LLVMInt32TypeInContext(self.context);
            let addr_a = self.get_value("A");
            let addr_x = self.get_value("X");
            let addr_mem = self.get_value("MEM");
            let a = llvm::core::LLVMBuildLoad(self.builder, addr_a, cstr!("A"));
            let x = llvm::core::LLVMBuildLoad(self.builder, addr_x, cstr!("X"));
            let k = llvm::core::LLVMConstInt(ty_i32, insn.k as _, 1);
            let data = llvm::core::LLVMGetParam(self.get_function("main"), 0);
            (addr_a, addr_x, addr_mem, a, x, k, data, ty_i32)
        };

        match bpf_class(insn.code) {
            BPF_RET => match bpf_rval(insn.code) {
                BPF_A => unsafe {
                    llvm::core::LLVMBuildRet(self.builder, a);
                },
                BPF_K => unsafe {
                    llvm::core::LLVMBuildRet(self.builder, x);
                },
                _ => panic!("InvalidRval"),
            },

            BPF_LD => match (bpf_size(insn.code), bpf_mode(insn.code)) {
                // A = data[k(+x)]
                (_, n @ BPF_ABS) | (_, n @ BPF_IND) => unsafe {
                    let v = if n == BPF_IND {
                        llvm::core::LLVMBuildAdd(self.builder, x, k, cstr!())
                    } else {
                        k
                    };
                    let f = match bpf_size(insn.code) {
                        BPF_W => self.get_function("ldw"),
                        BPF_H => self.get_function("ldh"),
                        BPF_B => self.get_function("ldb"),
                        _ => unreachable!(),
                    };
                    let v = llvm::core::LLVMBuildCall(
                        self.builder,
                        f,
                        [data, v].as_ptr() as *mut _,
                        2,
                        cstr!(),
                    );
                    llvm::core::LLVMBuildStore(self.builder, v, addr_a);
                },
                (BPF_W, BPF_LEN) => {
                    panic!("not supported");
                }
                (BPF_W, BPF_IMM) => unsafe {
                    // A = insn.k
                    llvm::core::LLVMBuildStore(self.builder, k, addr_a);
                },
                (BPF_W, BPF_MEM) => unsafe {
                    // A = mem[k];
                    let idx = k;
                    let p = llvm::core::LLVMBuildInBoundsGEP(
                        self.builder,
                        addr_mem,
                        [idx].as_ptr() as *mut _,
                        1,
                        cstr!(),
                    );
                    let v = llvm::core::LLVMBuildLoad(self.builder, p, cstr!());
                    llvm::core::LLVMBuildStore(self.builder, v, addr_a);
                },
                _ => panic!("InvalidLdInstruction"),
            },

            BPF_LDX => match (bpf_size(insn.code), bpf_mode(insn.code)) {
                (BPF_W, BPF_LEN) => {
                    panic!("not supported");
                }
                // X = (data[k] & 0xf) << 2
                (BPF_B, BPF_MSH) => unsafe {
                    let v = llvm::core::LLVMBuildCall(
                        self.builder,
                        self.get_function("msh"),
                        [data, k].as_ptr() as *mut _,
                        2,
                        cstr!(),
                    );
                    llvm::core::LLVMBuildStore(self.builder, v, addr_x);
                },
                // X = insn.k
                (BPF_W, BPF_IMM) => unsafe {
                    llvm::core::LLVMBuildStore(self.builder, k, addr_x);
                },
                // X = mem[k]
                (BPF_W, BPF_MEM) => unsafe {
                    let idx = k;
                    let p = llvm::core::LLVMBuildInBoundsGEP(
                        self.builder,
                        addr_mem,
                        [idx].as_ptr() as *mut _,
                        1,
                        cstr!(),
                    );
                    let v = llvm::core::LLVMBuildLoad(self.builder, p, cstr!());
                    llvm::core::LLVMBuildStore(self.builder, v, addr_x);
                },
                _ => panic!("InvalidLdInstruction"),
            },

            n @ BPF_ST | n @ BPF_STX => unsafe {
                // mem[k] = a or x
                let idx = k;
                let p = llvm::core::LLVMBuildInBoundsGEP(
                    self.builder,
                    addr_mem,
                    [idx].as_ptr() as *mut _,
                    1,
                    cstr!(),
                );
                let v = if n == BPF_ST { a } else { x };
                llvm::core::LLVMBuildStore(self.builder, v, p);
            },

            BPF_JMP => if bpf_op(insn.code) == BPF_JA {
                unsafe {
                    llvm::core::LLVMBuildBr(self.builder, bbs[insn.k as usize + idx + 1]);
                }
            } else {
                let jt_bb = bbs[insn.jt as usize + idx + 1];
                let jf_bb = bbs[insn.jt as usize + idx + 1];

                let src = match bpf_src(insn.code) {
                    BPF_K => k,
                    BPF_X => x,
                    _ => panic!("InvalidSrc"),
                };

                let cond = if bpf_op(insn.code) == BPF_JSET {
                    // a & src > 0
                    unsafe {
                        let a = llvm::core::LLVMBuildAnd(self.builder, a, src, cstr!());
                        let zero = llvm::core::LLVMConstInt(ty_i32, 0, 1);
                        llvm::core::LLVMBuildICmp(
                            self.builder,
                            llvm::LLVMIntPredicate::LLVMIntSGT,
                            a,
                            zero,
                            cstr!(),
                        )
                    }
                } else {
                    let pred = match bpf_op(insn.code) {
                        BPF_JGT => llvm::LLVMIntPredicate::LLVMIntSGT,
                        BPF_JGE => llvm::LLVMIntPredicate::LLVMIntSGE,
                        BPF_JEQ => llvm::LLVMIntPredicate::LLVMIntEQ,
                        _ => panic!("InvalidJmpCondition"),
                    };

                    unsafe { llvm::core::LLVMBuildICmp(self.builder, pred, a, src, cstr!()) }
                };
                unsafe {
                    llvm::core::LLVMBuildCondBr(self.builder, cond, jt_bb, jf_bb);
                }
            },

            BPF_ALU => {
                let a = if bpf_op(insn.code) == BPF_NEG {
                    unsafe { llvm::core::LLVMBuildNeg(self.builder, a, cstr!()) }
                } else {
                    let v = match bpf_src(insn.code) {
                        BPF_K => k,
                        BPF_X => x,
                        _ => panic!("InvalidSrc"),
                    };

                    match bpf_op(insn.code) {
                        BPF_ADD => unsafe { llvm::core::LLVMBuildAdd(self.builder, a, v, cstr!()) },
                        BPF_SUB => unsafe { llvm::core::LLVMBuildSub(self.builder, a, v, cstr!()) },
                        BPF_MUL => unsafe { llvm::core::LLVMBuildMul(self.builder, a, v, cstr!()) },
                        BPF_DIV => unsafe {
                            llvm::core::LLVMBuildSDiv(self.builder, a, v, cstr!())
                        },
                        BPF_MOD => unsafe {
                            llvm::core::LLVMBuildSRem(self.builder, a, v, cstr!())
                        },
                        BPF_AND => unsafe { llvm::core::LLVMBuildAnd(self.builder, a, v, cstr!()) },
                        BPF_OR => unsafe { llvm::core::LLVMBuildOr(self.builder, a, v, cstr!()) },
                        BPF_XOR => unsafe { llvm::core::LLVMBuildXor(self.builder, a, v, cstr!()) },
                        BPF_LSH => unsafe { llvm::core::LLVMBuildShl(self.builder, a, v, cstr!()) },
                        BPF_RSH => unsafe {
                            llvm::core::LLVMBuildAShr(self.builder, a, v, cstr!())
                        },
                        _ => panic!("InvalidAluOp"),
                    }
                };
                unsafe {
                    llvm::core::LLVMBuildStore(self.builder, a, addr_a);
                }
            }

            BPF_MISC => match bpf_miscop(insn.code) {
                BPF_TAX => unsafe {
                    llvm::core::LLVMBuildStore(self.builder, a, addr_x);
                },
                BPF_TXA => unsafe {
                    llvm::core::LLVMBuildStore(self.builder, x, addr_a);
                },
                _ => panic!("InvalidMiscOp"),
            },
            _ => panic!("InvalidInstruction"),
        }
    }

    // optimization
    // based on optimization code of https://bitbucket.org/tari/merthc

    // compile program
    pub fn jit_compile(&mut self) -> Result<(), String> {
        unsafe {
            llvm::execution_engine::LLVMLinkInMCJIT();
            let mut engine: LLVMExecutionEngineRef = mem::uninitialized();
            let mut err_msg: *mut i8 = mem::uninitialized();
            let mut options: LLVMMCJITCompilerOptions = mem::uninitialized();
            let options_size = mem::size_of::<LLVMMCJITCompilerOptions>();
            llvm::execution_engine::LLVMInitializeMCJITCompilerOptions(&mut options, options_size);
            options.OptLevel = 0;
            let result_code = llvm::execution_engine::LLVMCreateMCJITCompilerForModule(
                &mut engine,
                self.module,
                &mut options,
                options_size,
                &mut err_msg,
            );
            if result_code != 0 {
                return Err(
                    std::ffi::CStr::from_ptr(err_msg)
                        .to_string_lossy()
                        .into_owned(),
                );
            }

            let func_addr = llvm::execution_engine::LLVMGetFunctionAddress(engine, cstr!("main"));
            let func: Func = mem::transmute(func_addr);
            self.engine = Some(engine);
            self.jit_func = Some(func);
            Ok(())
        }
    }

    pub unsafe fn run_jit_func(&self, data: &mut [i8]) -> i32 {
        self.jit_func.expect("compile program first")(data.as_ptr() as *mut i8)
    }
}


impl Drop for Converter {
    fn drop(&mut self) {
        unsafe {
            // it gives error... why??
            // self.engine
            //     .map(|e| llvm::execution_engine::LLVMDisposeExecutionEngine(e));
            llvm::core::LLVMDisposeBuilder(self.builder);
            llvm::core::LLVMDisposeModule(self.module);
            llvm::core::LLVMContextDispose(self.context);
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn convert(insns: &[BpfInsn]) -> Result<String, String> {
        let mut converter = Converter::new();
        let ir = converter.convert(insns);
        converter.dump_module();
        ir
    }

    #[test]
    fn convert_all_insn() {
        let insns = [
            BpfInsn::new(BPF_LD_W_ABS, 0, 0, 0),
            BpfInsn::new(BPF_LD_H_ABS, 0, 0, 0),
            BpfInsn::new(BPF_LD_B_ABS, 0, 0, 0),
            BpfInsn::new(BPF_LD_W_IND, 0, 0, 0),
            BpfInsn::new(BPF_LD_H_IND, 0, 0, 0),
            BpfInsn::new(BPF_LD_B_IND, 0, 0, 0),
            BpfInsn::new(BPF_LDX_B_MSH, 0, 0, 0),
            BpfInsn::new(BPF_LD_IMM, 0, 0, 0),
            BpfInsn::new(BPF_LDX_IMM, 0, 0, 0),
            BpfInsn::new(BPF_LD_MEM, 0, 0, 0),
            BpfInsn::new(BPF_LDX_MEM, 0, 0, 0),
            BpfInsn::new(BPF_ST, 0, 0, 0),
            BpfInsn::new(BPF_STX, 0, 0, 0),
            BpfInsn::new(BPF_JMP_JA, 0, 0, 0),
            BpfInsn::new(BPF_JGT_K, 0, 0, 0),
            BpfInsn::new(BPF_JGE_K, 0, 0, 0),
            BpfInsn::new(BPF_JEQ_K, 0, 0, 0),
            BpfInsn::new(BPF_JSET_K, 0, 0, 0),
            BpfInsn::new(BPF_JGT_X, 0, 0, 0),
            BpfInsn::new(BPF_JGE_X, 0, 0, 0),
            BpfInsn::new(BPF_JEQ_X, 0, 0, 0),
            BpfInsn::new(BPF_JSET_X, 0, 0, 0),
            BpfInsn::new(BPF_ADD_X, 0, 0, 0),
            BpfInsn::new(BPF_SUB_X, 0, 0, 0),
            BpfInsn::new(BPF_MUL_X, 0, 0, 0),
            BpfInsn::new(BPF_DIV_X, 0, 0, 0),
            BpfInsn::new(BPF_MOD_X, 0, 0, 0),
            BpfInsn::new(BPF_AND_X, 0, 0, 0),
            BpfInsn::new(BPF_OR_X, 0, 0, 0),
            BpfInsn::new(BPF_XOR_X, 0, 0, 0),
            BpfInsn::new(BPF_LSH_X, 0, 0, 0),
            BpfInsn::new(BPF_RSH_X, 0, 0, 0),
            BpfInsn::new(BPF_ADD_K, 0, 0, 0),
            BpfInsn::new(BPF_SUB_K, 0, 0, 0),
            BpfInsn::new(BPF_MUL_K, 0, 0, 0),
            BpfInsn::new(BPF_DIV_K, 0, 0, 0),
            BpfInsn::new(BPF_MOD_K, 0, 0, 0),
            BpfInsn::new(BPF_AND_K, 0, 0, 0),
            BpfInsn::new(BPF_OR_K, 0, 0, 0),
            BpfInsn::new(BPF_XOR_K, 0, 0, 0),
            BpfInsn::new(BPF_LSH_K, 0, 0, 0),
            BpfInsn::new(BPF_RSH_K, 0, 0, 0),
            BpfInsn::new(BPF_ALU_NEG, 0, 0, 0),
            BpfInsn::new(BPF_MISC_TAX, 0, 0, 0),
            BpfInsn::new(BPF_MISC_TXA, 0, 0, 0),
            BpfInsn::new(BPF_RET_A, 0, 0, 0),
            BpfInsn::new(BPF_RET_K, 0, 0, 0),
        ];
        let ir = convert(&insns);
        assert!(ir.is_ok());
    }

    #[test]
    fn jit_test() {
        // a = 10; x = 20; x += a; ret x;
        let insns = [
            BpfInsn::new(BPF_LD_IMM, 0, 0, 10),
            BpfInsn::new(BPF_LDX_IMM, 0, 0, 20),
            BpfInsn::new(BPF_ADD_X, 0, 0, 0),
            BpfInsn::new(BPF_RET_A, 0, 0, 0),
        ];


        let mut converter = Converter::new();
        let ir = converter.convert(&insns);
        let r = converter.jit_compile();
        assert!(ir.is_ok());
        assert!(r.is_ok());
        unsafe {
            assert_eq!(converter.run_jit_func(&mut []), 30);
        }
    }
}
