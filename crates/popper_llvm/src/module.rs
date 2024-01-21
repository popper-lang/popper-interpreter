use llvm_sys::prelude::{
    LLVMModuleRef,
    LLVMMemoryBufferRef
};
use llvm_sys::core::{
    LLVMModuleCreateWithNameInContext,
    LLVMDumpModule,
    LLVMAppendModuleInlineAsm,
    LLVMAddFunction,
    LLVMCreateMemoryBufferWithContentsOfFile
};
use llvm_sys::bit_reader::{
    LLVMParseBitcodeInContext2
};
use llvm_sys::linker::LLVMLinkModules2;

use crate::context::Context;
use crate::types::function_types::FunctionType;
use crate::value::function_value::FunctionValue;

#[derive(Copy, Clone)]
pub struct Module {
    pub(crate) module: LLVMModuleRef,
    pub(crate) context: Context,
}

impl Module {

    pub fn from_bc_file(path: &str, context: Context) -> Self {
        let path = std::ffi::CString::new(path).unwrap();
        let module;
        let mut err = std::ptr::null_mut();
        unsafe {
            let mut buf_uninit = std::mem::MaybeUninit::uninit();
            let mut mod_uninit = std::mem::MaybeUninit::uninit();
            let buf: LLVMMemoryBufferRef;

            let res_buf =  LLVMCreateMemoryBufferWithContentsOfFile(path.as_ptr(), buf_uninit.as_mut_ptr(), &mut err);
            if res_buf != 0 {
                assert!(!err.is_null());
                panic!("Failed to load bitcode: {}", std::ffi::CStr::from_ptr(err).to_str().unwrap());
            }

            buf = buf_uninit.assume_init();

            let result = LLVMParseBitcodeInContext2(context.context, buf, mod_uninit.as_mut_ptr());
            if result != 0 {
                panic!("Failed to load bitcode");
            }

            module = mod_uninit.assume_init();
        }
        Self { module, context }
    }

    pub fn new(name: &str, context: Context) -> Self {
        let name = std::ffi::CString::new(name).unwrap();
        let module = unsafe {
            LLVMModuleCreateWithNameInContext(name.as_ptr(), context.context)
        };
        Self { module, context }
    }

    pub fn link(&self, other: &Module) {
        let result = unsafe { LLVMLinkModules2(self.module, other.module) };
        if result != 0 {
            panic!("Failed to link modules");
        }
    }

    pub fn get_context(&self) -> Context {
        self.context.clone()
    }



    pub fn dump(&self) {
        unsafe { LLVMDumpModule(self.module) }
    }

    pub fn push_asm(&self, asm_code: String)  {
        let asm_code = std::ffi::CString::new(asm_code).unwrap();
        let len = asm_code.clone().into_bytes().len();
        unsafe {
            LLVMAppendModuleInlineAsm(self.module, asm_code.as_ptr(), len);
        }
    }

    pub fn add_function(&self, name: &str, function_type: FunctionType) -> FunctionValue {
        let name = std::ffi::CString::new(name).unwrap();
        let function = unsafe {
            LLVMAddFunction(self.module, name.as_ptr(), function_type.get_type_ref())
        };
        FunctionValue::new_llvm_ref(function)
    }

}