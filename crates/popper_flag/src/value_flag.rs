use std::collections::HashMap;
use std::fmt::Display;
use popper_ast::{Type, TypeKind};


#[derive(PartialEq, Clone, Debug)]
pub enum ValueFlag {
    Integer,
    Float,
    String,
    Boolean,
    None,
    Array(Box<ValueFlag>),
    Function(Vec<ValueFlag>, Box<ValueFlag>),
    Module(HashMap<String, String>)
}

impl Display for ValueFlag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValueFlag::Integer => write!(f, "int"),
            ValueFlag::Float => write!(f, "float"),
            ValueFlag::String => write!(f, "string"),
            ValueFlag::Boolean => write!(f, "bool"),
            ValueFlag::None => write!(f, "unit"),
            ValueFlag::Array(t) => write!(f, "[{}]", t.to_string()),
            ValueFlag::Function(args, returntype) => {
                let mut args_string = String::new();
                for arg in args {
                    args_string.push_str(&arg.to_string());
                    args_string.push_str(", ");
                }
                args_string.pop();
                args_string.pop();
                write!(f, "func({}): {}", args_string, returntype.to_string())
            },
            ValueFlag::Module(hash) => write!(f, "module({:?})", hash)
        }
    }
}

impl ValueFlag {
    pub fn from_ty_kind(ty: TypeKind) -> Self {
        match ty {
            TypeKind::String => ValueFlag::String,
            TypeKind::Bool => ValueFlag::Boolean,
            TypeKind::Int => ValueFlag::Integer,
            TypeKind::Unit => ValueFlag::None,
            TypeKind::Array(ty, _) => ValueFlag::Array(
                Box::new(
                    Self::from_ty(*ty)
                )
            ),
            TypeKind::Function(args, ret) => ValueFlag::Function(args
                .iter()
                .cloned()
                .map(Self::from_ty).collect(),
                                                                 Box::new(Self::from_ty(*ret))
            ),
            _ => unimplemented!()

        }
    }

    pub fn from_ty(ty: Type) -> Self {
        Self::from_ty_kind(ty.type_kind)
    }
}