use mirage::backend::codegen_llvm::Compiler as LLVMCompiler;
use mirage::frontend::builder::{BasicBlock, Builder};
use mirage::frontend::module::Module;
use mirage::frontend::object::label::{Command, LabelBodyInstr, Value};
use mirage::frontend::object::meta::Flag;
use mirage::frontend::object::stringify::Stringify;
use mirage::frontend::object::{function::*, StructValue};
use mirage::frontend::object::{MirageObject, MirageTypeEnum, MirageValueEnum};
use std::collections::HashMap;
mod tag;

use tag::*;

#[derive(Debug, Clone)]
pub struct Compiler {
    stmts: Vec<popper_ast::Statement>,
    env: HashMap<String, Tagged<MirageValueEnum>>,
    current_basic_block: Option<BasicBlock>,
    current_function: Option<FunctionValue>,
    module: Module,
    builder: Builder,
    is_not_loadable: bool,
    struct_env: HashMap<String, (MirageTypeEnum, popper_ast::StructStmt)>,
}

impl Compiler {
    pub fn new(stmts: Vec<popper_ast::Statement>, filename: &str) -> Self {
        let module = Module::new(filename.to_string());
        Self {
            stmts,
            env: HashMap::new(),
            module: module.clone(),
            builder: Builder::new(module),
            current_basic_block: None,
            current_function: None,
            is_not_loadable: false,
            struct_env: HashMap::new(),
        }
    }

    pub fn popper_ty_to_mirage_ty(&self, ty: popper_ast::Type) -> MirageTypeEnum {
        match ty.type_kind {
            popper_ast::TypeKind::Int => MirageTypeEnum::type_int32().into(),
            popper_ast::TypeKind::Float => MirageTypeEnum::type_float32().into(),
            popper_ast::TypeKind::String(length) => {
                MirageTypeEnum::type_array(MirageTypeEnum::type_int8().into(), length as usize)
                    .into()
            }
            popper_ast::TypeKind::Bool => MirageTypeEnum::type_int8().into(),
            popper_ast::TypeKind::List(t, u) => {
                MirageTypeEnum::type_array(self.popper_ty_to_mirage_ty(*t), u).into()
            }
            popper_ast::TypeKind::Pointer(t) => {
                MirageTypeEnum::type_ptr(self.popper_ty_to_mirage_ty(*t)).into()
            }
            popper_ast::TypeKind::Struct(s) => self.struct_env.get(&s).unwrap().0.clone(),
            popper_ast::TypeKind::StructInstance(s) => self.struct_env.get(&s).unwrap().0.clone(),
            e => todo!("{:?}", e),
        }
    }

    pub fn compile(&mut self) {
        for stmt in self.stmts.clone() {
            self.compile_statement(stmt);
        }
    }

    pub fn compile_statement(&mut self, stmt: popper_ast::Statement) {
        match stmt {
            popper_ast::Statement::Function(f) => {
                self.compile_function(&f);
            }
            popper_ast::Statement::Expression(expr) => {
                self.compile_expr(expr);
            }
            popper_ast::Statement::Extern(ext) => {
                for sign in ext.signs {
                    let args = sign
                        .arguments
                        .args
                        .iter()
                        .map(|x| self.popper_ty_to_mirage_ty(x.ty.clone()))
                        .collect();
                    let return_ty = self.popper_ty_to_mirage_ty(sign.return_type.clone());
                    let fn_ty = FunctionType::new(args, return_ty, sign.is_var_args);
                    self.builder.build_extern(sign.name.clone(), fn_ty);
                }
            }
            popper_ast::Statement::Let(l) => {
                let val = self.compile_expr(l.value);
                self.env.insert(l.name.name, val);
            }
            popper_ast::Statement::Return(r) => {
                let val = self.compile_expr(*r.expression.unwrap()).value;
                let basic_block = self.current_basic_block.as_mut().unwrap();

                basic_block.build_ret(val).unwrap();
            }
            popper_ast::Statement::Assign(a) => {
                self.is_not_loadable = true;
                let n = self
                    .compile_expr(a.name)
                    .value
                    .expect_register_value()
                    .unwrap();
                self.is_not_loadable = false;
                let v = self.compile_expr(a.value).value;
                let basic_block = self.current_basic_block.as_mut().unwrap();
                basic_block.build_store(n, MirageObject::from(v)).unwrap();
            }
            popper_ast::Statement::Struct(s) => {
                let mut fields = Vec::new();
                for field in s.fields.iter() {
                    fields.push(self.popper_ty_to_mirage_ty(field.ty.clone()));
                }
                let ty = MirageTypeEnum::type_struct(fields);
                self.struct_env.insert(s.name.clone(), (ty.into(), s));
            }
            e => todo!("{:?}", e),
        }
    }

    pub fn compile_function(&mut self, f: &popper_ast::Function) {
        let mut args = Vec::new();

        for arg in f.arguments.args.iter() {
            args.push(self.popper_ty_to_mirage_ty(arg.ty.clone()));
        }

        let return_ty = self.popper_ty_to_mirage_ty(f.returntype.clone());

        let fn_ty = FunctionType::new(args, return_ty, f.is_var_args);

        let mut fn_value = fn_ty.fn_value(f.name.clone());

        for (avalue, aname) in fn_value.get_args().iter().zip(f.arguments.args.iter()) {
            self.env
                .insert(aname.name.clone(), Tagged::void(avalue.clone()));
        }

        self.current_function = Some(fn_value.clone());

        self.current_basic_block = Some(self.builder.new_basic_block("entry"));

        for stmt in f.body.clone() {
            self.compile_statement(stmt);
        }

        self.builder.join_function(
            &mut fn_value,
            self.current_basic_block.as_ref().unwrap().clone(),
        );

        self.builder.build_function(fn_value.clone());
    }

    fn compile_expr(&mut self, expr: popper_ast::Expression) -> Tagged<MirageValueEnum> {
        Tagged::void(match expr {
            popper_ast::Expression::Call(call) => {
                let args: Vec<_> = call
                    .arguments
                    .iter()
                    .map(|x| self.compile_expr(x.clone()).value)
                    .collect();
                let basic_block = self.current_basic_block.as_mut().unwrap();
                basic_block.build_call(call.name, args).unwrap()
            }
            popper_ast::Expression::Constant(constant) => match constant {
                popper_ast::Constant::Int(i) => {
                    let ty = MirageTypeEnum::type_int32();
                    ty.const_value(i.value as i32).to_value_enum()
                }

                popper_ast::Constant::Float(f) => {
                    let ty = MirageTypeEnum::type_float32();
                    ty.const_value(f.value as f32).to_value_enum()
                }

                popper_ast::Constant::Null(_n) => todo!(),
                popper_ast::Constant::List(l) => {
                    let mut values = Vec::new();
                    for v in l.value.iter() {
                        values.push(self.compile_expr(v.clone()).value);
                    }
                    if values.is_empty() {
                        panic!("A list can't be 0")
                    }

                    let ty = values.first().unwrap().get_type();
                    let ty = MirageTypeEnum::type_array(ty, values.len());
                    ty.const_value(values).to_mirage_value()
                }

                popper_ast::Constant::StringLiteral(s) => {
                    let ty =
                        MirageTypeEnum::type_array(MirageTypeEnum::type_int8().into(), s.len());
                    let value = ty
                        .const_value(
                            s.value
                                .as_bytes()
                                .to_vec()
                                .iter()
                                .map(|x| {
                                    let ty = MirageTypeEnum::type_int8();
                                    ty.const_value(*x as i8).to_value_enum()
                                })
                                .collect(),
                        )
                        .to_mirage_value();
                    self.builder.build_global(MirageObject::from(value))
                }

                popper_ast::Constant::Ident(id) => return self.env.get(&id.name).unwrap().clone(),

                e => todo!("{:?}", e),
            },
            popper_ast::Expression::Reference(r) => {
                let val = self.compile_expr(*r.expr);
                if self.is_not_loadable {
                    return val;
                }
                let basic_block = self.current_basic_block.as_mut().unwrap();
                basic_block.build_ref(val.value).unwrap()
            }
            popper_ast::Expression::BinOp(bin_op) => {
                let l = self.compile_expr(*bin_op.lhs).value;
                let r = self.compile_expr(*bin_op.rhs).value;
                let basic_block = self.current_basic_block.as_mut().unwrap();

                match bin_op.op {
                    popper_ast::BinOpKind::Add => basic_block
                        .build_int_add(l.expect_int_value().unwrap(), r.expect_int_value().unwrap())
                        .unwrap(),
                    popper_ast::BinOpKind::Sub => basic_block
                        .build_int_sub(l.expect_int_value().unwrap(), r.expect_int_value().unwrap())
                        .unwrap(),
                    e => todo!("{:?}", e),
                }
            }
            popper_ast::Expression::Deref(d) => {
                let val = self.compile_expr(*d.expr);
                if self.is_not_loadable {
                    return val;
                }
                let elt = val.get_type().expect_ptr_type().element_ty;
                let basic_block = self.current_basic_block.as_mut().unwrap();
                basic_block.build_load(*elt, val.value).unwrap()
            }
            popper_ast::Expression::StructInstance(s) => {
                let mut values = Vec::new();
                for v in s.fields.iter() {
                    values.push(self.compile_expr(v.value.clone()).value);
                }
                let ty = self.struct_env.get(&s.name).unwrap().0.clone();
                return MirageValueEnum::Struct(StructValue::new(ty.expect_struct_type(), values))
                    .tag(s.name.clone());
            }
            popper_ast::Expression::StructFieldAccess(s) => {
                let struct_ = self.compile_expr(*s.name);
                let name = struct_.tag;
                let ast_struct = self.struct_env.get(&name).unwrap().1.clone();
                let struct_ = struct_.value;
                let struct_ty = struct_.get_type().expect_struct_type();

                let index = ast_struct
                    .fields
                    .iter()
                    .position(|x| x.name == s.field)
                    .unwrap();

                let field_ty = struct_ty.fields[index].clone();
                let index = MirageTypeEnum::type_int32()
                    .const_value(index as i32)
                    .to_value_enum();
                let basic_block = self.current_basic_block.as_mut().unwrap();
                let r = basic_block
                    .build_getelementptr(field_ty, struct_ty.into(), struct_, vec![index])
                    .unwrap();

                r
            }

            _ => todo!(),
        })
    }

    pub fn print_to_string(&self) -> String {
        self.builder
            .asts
            .iter()
            .map(|x| x.to_string() + "\n")
            .collect()
    }

    pub fn compile_to_llvm(&mut self, debug: bool) -> String {
        let mut compiler = LLVMCompiler::new(self.builder.asts.clone(), debug).unwrap();

        compiler.compile();

        compiler.print_to_string()
    }
}
