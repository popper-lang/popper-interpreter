use std::process::Output;

use popper_codegen::Compiler;
use popper_parser::parser::parse;
use popper_ast::Statement;
// use popper_codegen::compiler::Compiler;
use popper_semantic_analyzer::analyze;
use popper_error::generate_color;
use std::path::Path;
// use popper_inkwell::compiler::Compiler as InkwellCompiler;
// use popper_inkwell::Context;

///
/// get_ast is used to get ast from input
/// # Arguments
/// * `input` - input string
/// * `file` - file name
///
/// return: `Option<Vec<Statement>>`
pub fn get_ast(input: &str, file: &str) -> Option<Vec<Statement>> {
    parse(input, file)
}


///
/// check_program is used to check program
/// # Arguments
/// * `ast` - ast
/// * `source` - source code
/// * `file_name` - file name
pub fn check_program(ast: Vec<Statement>, source: &str, file_name: &str) -> bool {
    let res = analyze(ast);

    let mut errs = Vec::new();
    for r in res  {
        match r {
            Ok(_) => {},
            Err(err) => {
                errs.push(err)
            }
        }
    }

    if errs.is_empty() {
        true
    } else {
        errs
            .into_iter()
            .for_each(|x| {
                x.report(
                    generate_color(),
                    source,
                    file_name,
                )
            })
        ;

        false
    }

}

// pub fn compile_to_inkwell_llvm<'a>(mir: Module) -> (String, Vec<String>) {
//     let context = Context::create();
//     let mut compiler = InkwellCompiler::new(mir, &context);
//     compiler.compile();
//     (compiler.build(), compiler.get_used_cdylibs())
// }

pub fn compile_to_mirage(ast: Vec<Statement>, file_name: &str) -> String {
    let mut compiler = Compiler::new(ast, file_name);
    compiler.compile();
    compiler.print_to_string()

}

pub fn compile_to_llvm(ast: Vec<Statement>, file_name: &str) -> String {
    let mut compiler = Compiler::new(ast, file_name);
    compiler.compile();
    compiler.compile_to_llvm()
}

pub fn execute_llvm(llvm: String, file_name: String, target_path: String, debug: bool) {
    use std::process::Command;

    let file_name = Path::new(Path::new(&file_name).file_name().unwrap().to_str().unwrap()).to_path_buf();
    let target_path = Path::new(&target_path).to_path_buf();
    let file_o = file_name.with_extension("o");
    let file_ll = file_name.with_extension("ll");
    let file_exe = file_name.with_extension("");

    if ! target_path.exists() {
        detail_output("mkdir", Command::new("mkdir")
            .arg(target_path.clone())
            .output()
            .expect("failed to execute process"),
        debug
        );

    }

    let file_o_path = target_path.join(file_o);
    let file_ll_path = target_path.join(file_ll);
    let file_exe_path = target_path.join(file_exe);

    if ! file_ll_path.exists() {
        Command::new("touch")
            .arg(file_ll_path.clone())
            .output()
            .expect("failed to execute process");
    }
    std::fs::write(file_ll_path.clone(), llvm).expect("Cannot write file");

    detail_output("llc", Command::new("llc")
        .arg(file_ll_path.clone())
        .arg("-filetype=obj")
        .arg("-o")
        .arg(file_o_path.clone())
        .output()
        .expect("failed to execute process `llc` "),
        debug
    );

    detail_output("clang", Command::new("clang")
        .arg(file_o_path.clone())
        .arg("-o")
        .arg(file_exe_path.clone())
        .output()
        .expect("failed to execute process `gcc` "),
        debug
    );



    detail_output("rm o", Command::new("rm")
        .arg(file_o_path.clone())
        .output()
        .expect("failed to execute process `rm` "),
        debug
    );

    Command::new("rm")
        .arg(file_ll_path.clone())
        .output()
        .expect("failed to execute process `rm` ");

    detail_output("exec", Command::new(
        format!("./{}", file_exe_path.clone().to_str().unwrap())
    )
        .output()
        .expect("failed to execute process your program "),
        debug
    );

}

fn detail_output(name: &str, output: Output, debug: bool) {
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    if debug {
        println!("({}) stdout: {}", name, stdout);
        println!("({}) stderr: {}", name, stderr);
    } else if ! stderr.is_empty() {
        println!("{}", stderr);
    } else if ! stdout.is_empty() {
        println!("{}", stdout);
    }

}
