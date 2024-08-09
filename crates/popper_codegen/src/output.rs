use crate::Compiler;
use mirage::backend::codegen_llvm::Compiler as LLVMCompiler;
use mirage::backend::output::*;

#[derive(Debug, Clone)]
pub struct Output {
    compiler: Compiler,
    llvm_compiler: LLVMCompiler,
}

impl Output {
    pub(crate) fn new(compiler: Compiler, llvm_compiler: LLVMCompiler) -> Self {
        Self {
            compiler,
            llvm_compiler,
        }
    }

    pub fn print_to_string(&self) -> String {
        self.compiler.print_to_string()
    }

    pub fn write_to_file(&self, path: &str) -> std::io::Result<()> {
        let content = self.print_to_string();
        std::fs::write(path, content)
    }

    pub fn print_llvm_to_string(&self) -> String {
        self.llvm_compiler.print_to_string()
    }

    pub fn write_llvm_to_file(&self, path: &str) -> std::io::Result<()> {
        let content = self.print_llvm_to_string();
        std::fs::write(path, content)
    }

    pub fn object_file(&mut self) -> ObjectOutput<'_> {
        self.llvm_compiler.object()
    }

    pub fn execution_engine<'a>(&'a mut self) -> impl ExecutionEngineOutput + 'a {
        self.llvm_compiler.execution_engine()
    }
}
