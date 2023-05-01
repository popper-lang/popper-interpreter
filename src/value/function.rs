use std::rc::Rc;

use super::{Object, Type};
use crate::ast::stmt::{Stmt, StmtType};

use crate::errors::{error, Error, ErrorType};
use crate::interpreter::environement::Environment;
use crate::interpreter::Interpreter;
use crate::value::callable::Callable;
use crate::value::{Implementation, RustValue, Var};



#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub declaration: Stmt,
}

impl Function {
    pub fn new(declaration: Stmt) -> Self {
        Function { declaration }
    }

    pub fn create_function(declaration: Stmt) -> Object {
        Object {
            type_: Type::Function,
            implementations: vec![
                Implementation::Call(Rc::new(Function::new(declaration)))
            ],
            value: RustValue::Function
        }
    }
}

impl Callable for Function {
    fn call(&self, _interpreter: &mut Interpreter, args: Vec<Object>, _file: &str) -> Object {
        println!("BREAK 7");
        let mut env = Environment::new(None);
        let mut new_interpreter = Interpreter::new();
        env = new_interpreter.env.clone();

        let mut i = 0;
        match &*self.declaration.stmt_type {
            StmtType::Function { args: params, name, body } => {
                println!("BREAK 8: {:#?}", params);
                env.define(name.lexeme.to_string(), Var {
                    value: Object {
                        type_: Type::Function,
                        implementations: vec![
                            Implementation::Call(Rc::new(Function::new(self.declaration.clone())))
                        ],
                        value: RustValue::Function
                    },
                    mutable: false,
                    type_: Type::Function
                });
                println!("BREAK 9");
                for arg in params {
                    println!("BREAK 10");
                    env.define(arg.clone(), Var {
                        value: args[i].clone(),
                        mutable: false,

                        type_: args[i].type_.clone()
                    });
                    println!("BREAK 11");
                    i += 1;
                }
                println!("BREAK 12");
                new_interpreter.env = env;
                println!("BREAK 12: {:#?}", body);
                let res = body.clone().accept(&mut new_interpreter);
                println!("BREAK 13");
                res
            },
            _ => {
                error!(ErrorType::TypeError, "Expected a function", 0..0, "".to_string());
                unreachable!()
            }
        }
    }
}


