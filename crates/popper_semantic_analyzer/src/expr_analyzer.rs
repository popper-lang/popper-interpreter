use popper_ast::*;

use popper_error::{
    diff_length_of_argument::DiffLengthOfArgument, namenotfound::NameNotFound,
    typemismatch::TypeMismatch, typenotspecified::TypeNotSpecified,
};
use popper_flag::{Environment, Flag, SymbolFlags, ValueFlag};

use popper_ast::visitor::ExprVisitor;
use popper_common::name_similarity::find_similar_name;
use popper_error::fieldnotfound::FieldNotFound;
use popper_error::Error;

#[derive(Clone)]
pub struct ExprAnalyzer {
    env: Environment,
    let_expected_value: Option<SymbolFlags>,
}

impl ExprAnalyzer {
    pub fn new(env: Environment) -> Self {
        Self {
            env,
            let_expected_value: None,
        }
    }

    pub(crate) fn set_let_expected_value(&mut self, value: SymbolFlags) {
        self.let_expected_value = Some(value);
    }

    pub fn get_type(&self, ty: Type) -> ValueFlag {
        match ty.type_kind {
            TypeKind::Bool => ValueFlag::Boolean,
            TypeKind::Float => ValueFlag::Float,
            TypeKind::Int => ValueFlag::Integer,
            TypeKind::String(size) => ValueFlag::String(size),
            TypeKind::List(ty, l) => ValueFlag::List(Box::new(self.get_type(*ty)), l),
            TypeKind::Function(args, returnty, is_var_args) => {
                let mut args_type = Vec::new();
                for arg in args {
                    args_type.push(self.get_type(arg));
                }
                ValueFlag::Function(args_type, Box::new(self.get_type(*returnty)), is_var_args)
            }
            TypeKind::Unit => ValueFlag::None,
            TypeKind::Pointer(ptr) => ValueFlag::Pointer(Box::new(self.get_type(*ptr))),
            TypeKind::Struct(name) => ValueFlag::Struct(name),
            _ => unimplemented!(),
        }
    }
}

impl ExprVisitor for ExprAnalyzer {
    type Output = SymbolFlags;
    type Error = Box<dyn Error>;

    fn visit_constant(&mut self, constant: Constant) -> Result<Self::Output, Self::Error> {
        match constant {
            Constant::Int(int) => Ok(SymbolFlags::new(int.span()).set_integer().clone()),
            Constant::Float(float) => Ok(SymbolFlags::new(float.span()).set_float().clone()),
            Constant::StringLiteral(string) => Ok(SymbolFlags::new(string.span())
                .set_string(string.value.len() as u32)
                .clone()),
            Constant::Bool(bool) => Ok(SymbolFlags::new(bool.span()).set_boolean().clone()),
            Constant::Ident(ref ident) => match self.env.get_variable(&ident.name) {
                Some(v) => Ok({ v.value.clone() }),
                None => {
                    let name_candidates = self.env.get_all_variables_name();

                    let similar_name =
                        find_similar_name(name_candidates.as_slice(), ident.name.as_str());

                    Err(Box::new(NameNotFound::new(
                        (ident.span, ident.name.clone()),
                        similar_name.cloned(),
                    )))
                }
            },
            Constant::List(l) => {
                let mut flags = SymbolFlags::new(l.span());
                let mut base_value_flag: Option<ValueFlag> = None;
                let mut base_span: Option<Span> = None;
                for expr in &l.value {
                    let flag = self.visit_expr(expr.clone())?;
                    let value_flag = flag.get_value().unwrap();
                    if base_value_flag.is_some()
                        && !base_value_flag.as_ref().unwrap().is_same(&value_flag)
                    {
                        return Err(Box::new(TypeMismatch::new(
                            (
                                base_span.unwrap(),
                                base_value_flag.clone().unwrap().to_string(),
                            ),
                            (flag.span(), value_flag.to_string()),
                        )));
                    }
                    base_value_flag = Some(value_flag);
                    base_span = Some(flag.span());
                }
                if let Some(value_flag) = base_value_flag {
                    flags.set_list(value_flag, l.value.len());
                } else if let Some(val) = self.let_expected_value.clone() {
                    if let ValueFlag::List(ty, size) = val.get_value().unwrap() {
                        if size != l.value.len() {
                            return Err(Box::new(TypeMismatch::new(
                                (
                                    self.let_expected_value.clone().unwrap().span(),
                                    self.let_expected_value
                                        .clone()
                                        .unwrap()
                                        .get_value()
                                        .unwrap()
                                        .to_string(),
                                ),
                                (l.span(), format!("[{}: {}]", ty, l.value.len())),
                            )));
                        }
                        flags.set_list(*ty, size);
                    } else {
                        return Err(Box::new(TypeMismatch::new(
                            (
                                self.let_expected_value.clone().unwrap().span(),
                                self.let_expected_value
                                    .clone()
                                    .unwrap()
                                    .get_value()
                                    .unwrap()
                                    .to_string(),
                            ),
                            (l.span(), format!("list of length {}", l.value.len())),
                        )));
                    }
                } else {
                    return Err(Box::new(TypeNotSpecified::new(
                        l.span(),
                        "array".to_string(),
                    )));
                }
                Ok(flags)
            }
            Constant::Null(null) => Ok(SymbolFlags::new(null.span()).set_none().clone()),
        }
    }

    fn visit_deref(&mut self, pointer: Deref) -> Result<Self::Output, Self::Error> {
        let flag = self.visit_expr(*pointer.expr)?;
        if !flag.is_pointer() {
            return Err(Box::new(TypeMismatch::new(
                (pointer.span, "pointer".to_string()),
                (flag.clone().span(), flag.get_value().unwrap().to_string()),
            )));
        }

        let flag = flag.get_minor_type().unwrap();
        Ok(SymbolFlags::new(pointer.span)
            .add_flag(Flag::Value(flag))
            .clone())
    }

    fn visit_reference(&mut self, reference: Reference) -> Result<Self::Output, Self::Error> {
        let flag = self.visit_expr(*reference.expr)?;
        Ok(SymbolFlags::new(reference.span)
            .set_pointer(flag.get_value().unwrap())
            .clone())
    }

    fn visit_va_arg(&mut self, va_arg: VaArg) -> Result<Self::Output, Self::Error> {
        let value_flag = ValueFlag::from_ty(va_arg.clone().ty);
        Ok(SymbolFlags::new(va_arg.span())
            .add_flag(Flag::Value(value_flag))
            .clone())
    }

    fn visit_bin_op(&mut self, bin_op: BinOp) -> Result<Self::Output, Self::Error> {
        let flag_lhs = self.visit_expr(*bin_op.lhs)?;
        let flag_rhs = self.visit_expr(*bin_op.rhs)?;
        if flag_lhs.is_same_value(flag_rhs.clone()) && bin_op.op.is_arithmetic() {
            Ok(flag_lhs)
        } else if flag_lhs.is_same_value(flag_rhs.clone()) && bin_op.op.is_comparison() {
            Ok(SymbolFlags::new(bin_op.span).set_boolean().clone())
        } else {
            Err(Box::new(TypeMismatch::new(
                (
                    flag_lhs.clone().span(),
                    flag_lhs.get_value().unwrap().to_string(),
                ),
                (
                    flag_rhs.clone().span(),
                    flag_rhs.get_value().unwrap().to_string(),
                ),
            )))
        }
    }

    fn visit_unary_op(&mut self, unary_op: UnaryOp) -> Result<Self::Output, Self::Error> {
        let flag_expr = self.visit_expr(*unary_op.expr)?;
        if unary_op.op == UnaryOpKind::Not {
            if flag_expr.clone().is_boolean() {
                Ok(flag_expr)
            } else {
                Err(Box::new(TypeMismatch::new(
                    (flag_expr.span, "boolean".to_string()),
                    (flag_expr.span, flag_expr.get_value().unwrap().to_string()),
                )))
            }
        } else if unary_op.op == UnaryOpKind::Neg {
            if flag_expr.is_integer() || flag_expr.is_float() {
                Ok(flag_expr)
            } else {
                Err(Box::new(TypeMismatch::new(
                    (flag_expr.span, "Integer or Float".to_string()),
                    (flag_expr.span, flag_expr.get_value().unwrap().to_string()),
                )))
            }
        } else {
            unreachable!()
        }
    }

    fn visit_group(&mut self, group: ParenGroup) -> Result<Self::Output, Self::Error> {
        self.visit_expr(*group.expr)
    }

    fn visit_call(&mut self, call: Call) -> Result<Self::Output, Self::Error> {
        let x = self.env.get_variable(&call.name);

        match x {
            Some(var) => match var.value.get_function() {
                Some((args, ret, is_var_args)) => {
                    let mut args_s = call
                        .arguments
                        .iter()
                        .map(|arg| self.clone().visit_expr(arg.clone()))
                        .collect::<Result<Vec<_>, _>>()?;
                    if is_var_args {
                        if args_s.len() < args.len() {
                            return Err(Box::new(DiffLengthOfArgument::new(
                                args.len(),
                                args_s.len(),
                                call.span,
                            )));
                        }

                        args_s = args_s.into_iter().take(args.len()).collect();
                    }
                    if args_s.len() != args.len() {
                        return Err(Box::new(DiffLengthOfArgument::new(
                            args.len(),
                            args_s.len(),
                            call.span,
                        )));
                    }
                    for (arg_get, arg_model) in args_s.iter().zip(args) {
                        let arg_get_value: ValueFlag = arg_get.get_value().unwrap();
                        let arg_model_value: ValueFlag = arg_model.clone();
                        if arg_get_value != arg_model_value {
                            return Err(Box::new(TypeMismatch::new(
                                (call.span, arg_model_value.to_string()),
                                (call.span, arg_get_value.to_string()),
                            )));
                        }
                    }
                    Ok(SymbolFlags::new(call.span).set_value(*ret.clone()).clone())
                }
                None => Err(Box::new(TypeMismatch::new(
                    (call.span, "function".to_string()),
                    (call.span, var.value.get_value().unwrap().to_string()),
                ))),
            },
            None => {
                let name_candidates = self.env.get_all_variables_name();
                let similar_name =
                    find_similar_name(name_candidates.as_slice(), call.name.as_str());

                Err(Box::new(NameNotFound::new(
                    (call.span, call.name.clone()),
                    similar_name.cloned(),
                )))
            }
        }
    }

    fn visit_struct_instance(
        &mut self,
        struct_instance: StructInstance,
    ) -> Result<Self::Output, Self::Error> {
        if !self.env.exist(struct_instance.name.clone()) {
            let name_candidates = self.env.get_all_variables_name();
            let similar_name =
                find_similar_name(name_candidates.as_slice(), struct_instance.name.as_str());

            return Err(Box::new(NameNotFound::new(
                (struct_instance.span, struct_instance.name.clone()),
                similar_name.cloned(),
            )));
        }

        let struct_model = self
            .env
            .get_variable(struct_instance.name.as_str())
            .unwrap();
        let struct_model_value = struct_model.value.get_value().unwrap();
        if let ValueFlag::Struct(ref name) = struct_model_value {
            let mut fields_s = Vec::new();

            for field in struct_instance.fields {
                fields_s.push((
                    field.name.clone(),
                    self.visit_expr(field.value)?.get_value().unwrap(),
                ))
            }
            let fields = self.env.get_struct(name).unwrap();
            let mut sorted_fields = fields.iter().collect::<Vec<_>>();
            sorted_fields.sort_by(|a, b| a.0.cmp(b.0));

            fields_s.sort_by(|a, b| a.0.cmp(&b.0));

            if fields.len() != fields_s.len() {
                return Err(Box::new(DiffLengthOfArgument::new(
                    fields.len(),
                    fields_s.len(),
                    struct_instance.span,
                )));
            }

            for (field_get, field_model) in fields_s.iter().zip(sorted_fields) {
                if field_get.0 != *field_model.0 {
                    return Err(Box::new(FieldNotFound::new(
                        field_get.0.clone(),
                        struct_instance.span,
                        None,
                    )));
                }

                if field_get.1 != *field_model.1 {
                    return Err(Box::new(TypeMismatch::new(
                        (struct_instance.span, field_model.1.to_string()),
                        (struct_instance.span, field_get.1.to_string()),
                    )));
                }
            }

            Ok(SymbolFlags::new(struct_instance.span)
                .set_value(struct_model_value)
                .clone())
        } else {
            Err(Box::new(TypeMismatch::new(
                (struct_instance.span, "struct".to_string()),
                (struct_instance.span, struct_model_value.to_string()),
            )))
        }
    }

    fn visit_struct_field_access(
        &mut self,
        struct_field_access: StructFieldAccess,
    ) -> Result<Self::Output, Self::Error> {
        let struct_model = self.visit_expr(*struct_field_access.name.clone())?;
        let mut struct_model_value = struct_model.get_value().unwrap();
        if struct_field_access.is_ptr {
            if !struct_model.is_pointer() {
                return Err(Box::new(TypeMismatch::new(
                    (struct_field_access.span, "pointer".to_string()),
                    (struct_field_access.span, struct_model_value.to_string()),
                )));
            }
            struct_model_value = struct_model.get_minor_type().unwrap();
        }
        if let ValueFlag::Struct(ref name) = struct_model_value {
            let s = self.env.get_struct(name).unwrap();
            match s.get(&struct_field_access.field) {
                Some(flag) => Ok(SymbolFlags::new(struct_field_access.span)
                    .set_value(flag.clone())
                    .clone()),
                None => {
                    let field_candidates = s.keys().cloned().collect::<Vec<_>>();
                    let similar_name = find_similar_name(
                        field_candidates.as_slice(),
                        struct_field_access.field.as_str(),
                    );

                    Err(Box::new(FieldNotFound::new(
                        struct_field_access.field.clone(),
                        struct_field_access.span,
                        similar_name.cloned(),
                    )))
                }
            }
        } else {
            Err(Box::new(TypeMismatch::new(
                (struct_field_access.span, "struct".to_string()),
                (struct_field_access.span, struct_model_value.to_string()),
            )))
        }
    }

    fn visit_index(&mut self, index: Index) -> Result<Self::Output, Self::Error> {
        let res = self.visit_expr(*index.value.clone())?;
        let ind = self.visit_expr(*index.index.clone())?;
        let list = res.get_list();

        if let Some(l) = list {
            if ind.is_integer() {
                Ok(SymbolFlags::new(index.span).set_value(l.0).clone())
            } else {
                Err(Box::new(TypeMismatch::new(
                    (index.index.span(), "int".to_string()),
                    (index.index.span(), ind.get_value().unwrap().to_string()),
                )))
            }
        } else {
            Err(Box::new(TypeMismatch::new(
                (index.value.span(), "[unknow: unknow]".to_string()),
                (index.value.span(), res.get_value().unwrap().to_string()),
            )))
        }
    }

    fn visit_expr(&mut self, expr: Expression) -> Result<Self::Output, Self::Error> {
        match expr {
            Expression::Constant(constant) => self.visit_constant(constant),
            Expression::BinOp(bin_op) => self.visit_bin_op(bin_op),
            Expression::UnaryOp(unary_op) => self.visit_unary_op(unary_op),
            Expression::Group(group) => self.visit_group(group),
            Expression::Call(call) => self.visit_call(call),
            Expression::StructInstance(struct_instance) => {
                self.visit_struct_instance(struct_instance)
            }
            Expression::StructFieldAccess(struct_field_access) => {
                self.visit_struct_field_access(struct_field_access)
            }
            Expression::Index(index) => self.visit_index(index),
            Expression::VaArg(va_arg) => self.visit_va_arg(va_arg),
            Expression::Reference(r) => self.visit_reference(r),
            Expression::Deref(p) => self.visit_deref(p),
        }
    }
}
