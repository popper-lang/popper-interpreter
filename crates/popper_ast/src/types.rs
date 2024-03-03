use crate::Span;
use std::collections::HashMap;

#[cfg_attr(feature = "extra-trait", derive(Debug, PartialEq))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone)]
/// a type
pub struct Type {
    pub span: Span,
    pub type_kind: TypeKind,
    pub generics: Vec<Type>,
}

impl Type {
    pub fn new(span: Span, type_kind: TypeKind, generics: Vec<Type>) -> Self {
        Self {
            span,
            type_kind,
            generics,
        }
    }
}

#[cfg_attr(feature = "extra-trait", derive(Debug, PartialEq))]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[derive(Clone)]
pub enum TypeKind {
    /// `(type,*)`
    Tuple(Vec<Type>),
    /// `[type]`
    List(Box<Type>, usize),
    /// `func(type,*) : type`
    Function(Vec<Type>, Box<Type>, bool),
    /// `*type`
    Pointer(Box<Type>),
    /// `()`
    Unit,
    /// `int`
    Int,
    /// `float`
    Float,
    /// `bool`
    Bool,
    /// `char`
    Char,
    /// `string`
    String(u32), // u32: size of string
    Struct(HashMap<String, Type>),
    /// `struct name
    StructInstance(String),
}

impl ToString for TypeKind {
    fn to_string(&self) -> String {
        match self.clone() {
            TypeKind::Tuple(tys) => format!(
                "({})",
                tys.iter()
                    .map(|ty| ty.type_kind.to_string())
                    .collect::<Vec<String>>()
                    .join(",")
            ),
            TypeKind::List(ty, size) => format!("[{}:{}]", ty.type_kind.to_string().clone(), size),
            TypeKind::Function(tys, ret, varargs) => format!(
                "func({}{}): {}",
                tys.iter()
                    .map(|t| t.type_kind.to_string())
                    .collect::<Vec<String>>()
                    .join(","),
                if varargs { "..." } else { "" },
                ret.type_kind.to_string()
            ),
            TypeKind::Pointer(ty) => format!("*{}", ty.type_kind.to_string()),
            TypeKind::Unit => String::from("()"),
            TypeKind::Int => String::from("int"),
            TypeKind::Float => String::from("float"),
            TypeKind::Bool => String::from("bool"),
            TypeKind::Char => String::from("char"),
            TypeKind::String(len) => format!("string:{}", len),
            TypeKind::Struct(fields) => {
                let mut fields_str = String::new();
                for (name, ty) in fields {
                    fields_str.push_str(&format!("{}: {},", name, ty.type_kind.to_string()));
                }
                format!("struct {{{}}}", fields_str)
            }
            TypeKind::StructInstance(name) => format!("struct {}", name),
        }
    }
}
