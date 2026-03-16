use crate::interpreter::environment::Environment;
use crate::interpreter::primitives;
use crate::interpreter::value::*;
use crate::parser::ast::*;
use crate::parser::ast::InterpolationPart;

pub struct Runtime {
    env: Environment,
}

enum ControlFlow {
    None,
    Return(LingotResult),
    Fail(String),
}

impl Runtime {
    pub fn new() -> Self {
        Runtime {
            env: Environment::new(),
        }
    }

    pub fn execute(&mut self, stmts: &[Stmt]) -> Result<LingotResult, String> {
        let mut last = LingotResult::void();
        for stmt in stmts {
            match self.exec_stmt(stmt)? {
                ControlFlow::None => {
                    last = LingotResult::void();
                }
                ControlFlow::Return(result) => return Ok(result),
                ControlFlow::Fail(msg) => return Ok(LingotResult::fail(msg)),
            }
        }
        Ok(last)
    }

    fn exec_stmt(&mut self, stmt: &Stmt) -> Result<ControlFlow, String> {
        match stmt {
            Stmt::Let { name, is_dyn, is_pub, value, .. } => {
                let val = self.eval_expr(value)?;
                self.env.define(name, val, *is_dyn, *is_pub)?;
                Ok(ControlFlow::None)
            }
            Stmt::FuncDecl { name, is_dyn, is_pub, params, body } => {
                let func = LingotValue::Func {
                    name: name.clone(),
                    params: params.clone(),
                    body: body.clone(),
                };
                self.env.define(name, func, *is_dyn, *is_pub)?;
                Ok(ControlFlow::None)
            }
            Stmt::Assign { name, value } => {
                let val = self.eval_expr(value)?;
                self.env.assign(name, val)?;
                Ok(ControlFlow::None)
            }
            Stmt::ExprStmt(expr) => {
                self.eval_expr(expr)?;
                Ok(ControlFlow::None)
            }
            Stmt::If { condition, then_branch, else_branch } => {
                let cond = self.eval_expr(condition)?;
                if self.is_truthy(&cond) {
                    self.exec_block(then_branch)
                } else if let Some(else_body) = else_branch {
                    self.exec_block(else_body)
                } else {
                    Ok(ControlFlow::None)
                }
            }
            Stmt::While { condition, body } => {
                loop {
                    let cond = self.eval_expr(condition)?;
                    if !self.is_truthy(&cond) {
                        break;
                    }
                    match self.exec_block(body)? {
                        ControlFlow::None => {}
                        cf => return Ok(cf),
                    }
                }
                Ok(ControlFlow::None)
            }
            Stmt::RepeatWhile { body, condition } => {
                loop {
                    match self.exec_block(body)? {
                        ControlFlow::None => {}
                        cf => return Ok(cf),
                    }
                    let cond = self.eval_expr(condition)?;
                    if !self.is_truthy(&cond) {
                        break;
                    }
                }
                Ok(ControlFlow::None)
            }
            Stmt::RepeatFor { var_name, iterable, body } => {
                let iter_val = self.eval_expr(iterable)?;
                let items = self.to_iterable(&iter_val)?;
                for item in items {
                    self.env.push_scope();
                    self.env.define(var_name, item, false, false)?;
                    let cf = self.exec_block(body)?;
                    self.env.pop_scope();
                    match cf {
                        ControlFlow::None => {}
                        cf => return Ok(cf),
                    }
                }
                Ok(ControlFlow::None)
            }
            Stmt::TryCatch { try_body, error_name, catch_body } => {
                match self.exec_block(try_body) {
                    Ok(ControlFlow::Fail(msg)) => {
                        self.env.push_scope();
                        self.env.define(error_name, LingotValue::Text(msg), false, false)?;
                        let cf = self.exec_block(catch_body)?;
                        self.env.pop_scope();
                        Ok(cf)
                    }
                    Err(msg) => {
                        self.env.push_scope();
                        self.env.define(error_name, LingotValue::Text(msg), false, false)?;
                        let cf = self.exec_block(catch_body)?;
                        self.env.pop_scope();
                        Ok(cf)
                    }
                    Ok(cf) => Ok(cf),
                }
            }
            Stmt::Return(expr) => {
                let val = self.eval_expr(expr)?;
                Ok(ControlFlow::Return(LingotResult::ok(val)))
            }
            Stmt::Fail(expr) => {
                let val = self.eval_expr(expr)?;
                Ok(ControlFlow::Fail(format!("{}", val)))
            }
            Stmt::Load { .. } => {
                // TODO: implement module loading
                Err("load is not yet implemented".to_string())
            }
        }
    }

    fn exec_block(&mut self, stmts: &[Stmt]) -> Result<ControlFlow, String> {
        self.env.push_scope();
        let mut result = ControlFlow::None;
        for stmt in stmts {
            result = self.exec_stmt(stmt)?;
            match &result {
                ControlFlow::Return(_) | ControlFlow::Fail(_) => {
                    self.env.pop_scope();
                    return Ok(result);
                }
                ControlFlow::None => {}
            }
        }
        self.env.pop_scope();
        Ok(result)
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<LingotValue, String> {
        match expr {
            Expr::NumberLit(n, is_float) => {
                if *is_float {
                    Ok(LingotValue::Number(LingotNumber::Float(*n)))
                } else {
                    Ok(LingotValue::Number(LingotNumber::from_f64(*n)))
                }
            }
            Expr::TextLit(s) => Ok(LingotValue::Text(s.clone())),
            Expr::BoolLit(b) => Ok(LingotValue::Bool(*b)),
            Expr::Identifier(name) => {
                let val = self.env.get(name)?;
                Ok(val.clone())
            }
            Expr::Binary { left, op, right } => {
                let l = self.eval_expr(left)?;
                let r = self.eval_expr(right)?;
                self.eval_binary(l, op, r)
            }
            Expr::Unary { op, expr } => {
                let val = self.eval_expr(expr)?;
                self.eval_unary(op, val)
            }
            Expr::Call { callee, args } => {
                // Handle builtins
                if let Expr::Identifier(name) = callee.as_ref() {
                    if let Some(result) = self.try_builtin(name, args)? {
                        return Ok(result);
                    }
                }

                let func = self.eval_expr(callee)?;
                let mut eval_args = Vec::new();
                for arg in args {
                    eval_args.push(self.eval_expr(arg)?);
                }
                self.call_function(func, eval_args)
            }
            Expr::MethodCall { object, method, args } => {
                let obj = self.eval_expr(object)?;
                let mut eval_args = Vec::new();
                for arg in args {
                    eval_args.push(self.eval_expr(arg)?);
                }
                self.call_method(obj, method, eval_args)
            }
            Expr::FieldAccess { object, field } => {
                let obj = self.eval_expr(object)?;
                self.access_field(obj, field)
            }
            Expr::ListLit(elements) => {
                let mut items = Vec::new();
                for elem in elements {
                    items.push(self.eval_expr(elem)?);
                }
                Ok(LingotValue::List(items))
            }
            Expr::Range { start, end } => {
                let s = self.eval_expr(start)?;
                let e = self.eval_expr(end)?;
                self.eval_range(s, e)
            }
            Expr::Interpolation(parts) => {
                let mut result = String::new();
                for part in parts {
                    match part {
                        InterpolationPart::Text(text) => result.push_str(text),
                        InterpolationPart::Expr(expr) => {
                            let val = self.eval_expr(expr)?;
                            result.push_str(&format!("{}", val));
                        }
                    }
                }
                Ok(LingotValue::Text(result))
            }
            _ => Err(format!("expression not yet implemented: {:?}", expr)),
        }
    }

    fn eval_binary(
        &self,
        left: LingotValue,
        op: &BinOp,
        right: LingotValue,
    ) -> Result<LingotValue, String> {
        match (left, right) {
            (LingotValue::Number(a), LingotValue::Number(b)) => {
                self.eval_number_binary(a, op, b)
            }
            (LingotValue::Text(a), LingotValue::Text(b)) => {
                match op {
                    BinOp::Add => Ok(LingotValue::Text(a + &b)),
                    BinOp::EqualEqual => Ok(LingotValue::Bool(a == b)),
                    BinOp::NotEqual => Ok(LingotValue::Bool(a != b)),
                    _ => Err(format!("operator {:?} not supported for Text", op)),
                }
            }
            // Text + anything = Text concatenation
            (LingotValue::Text(a), other) => {
                match op {
                    BinOp::Add => Ok(LingotValue::Text(format!("{}{}", a, other))),
                    _ => Err(format!("operator {:?} not supported for Text + {}", op, type_name(&other))),
                }
            }
            (other, LingotValue::Text(b)) => {
                match op {
                    BinOp::Add => Ok(LingotValue::Text(format!("{}{}", other, b))),
                    _ => Err(format!("operator {:?} not supported for {} + Text", op, type_name(&other))),
                }
            }
            (LingotValue::Bool(a), LingotValue::Bool(b)) => {
                match op {
                    BinOp::And => Ok(LingotValue::Bool(a && b)),
                    BinOp::Or => Ok(LingotValue::Bool(a || b)),
                    BinOp::EqualEqual => Ok(LingotValue::Bool(a == b)),
                    BinOp::NotEqual => Ok(LingotValue::Bool(a != b)),
                    _ => Err(format!("operator {:?} not supported for Bool", op)),
                }
            }
            (l, r) => Err(format!(
                "{} and {} do not support operator {:?}",
                type_name(&l), type_name(&r), op
            )),
        }
    }

    fn eval_number_binary(
        &self,
        a: LingotNumber,
        op: &BinOp,
        b: LingotNumber,
    ) -> Result<LingotValue, String> {
        match op {
            BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Mod => {
                let both_int = a.is_int() && b.is_int();
                let af = a.to_f64();
                let bf = b.to_f64();

                let result = match op {
                    BinOp::Add => af + bf,
                    BinOp::Sub => af - bf,
                    BinOp::Mul => af * bf,
                    BinOp::Div => {
                        if bf == 0.0 {
                            return Err("Division by zero".to_string());
                        }
                        if both_int {
                            return Ok(LingotValue::Number(LingotNumber::Int(
                                a.to_i64() / b.to_i64(),
                            )));
                        }
                        af / bf
                    }
                    BinOp::Mod => {
                        if bf == 0.0 {
                            return Err("Modulo by zero".to_string());
                        }
                        af % bf
                    }
                    _ => unreachable!(),
                };

                if both_int && matches!(op, BinOp::Add | BinOp::Sub | BinOp::Mul) {
                    Ok(LingotValue::Number(LingotNumber::Int(result as i64)))
                } else {
                    Ok(LingotValue::Number(LingotNumber::from_f64(result)))
                }
            }
            BinOp::EqualEqual => {
                let diff = (a.to_f64() - b.to_f64()).abs();
                Ok(LingotValue::Bool(diff < 1e-10))
            }
            BinOp::NotEqual => {
                let diff = (a.to_f64() - b.to_f64()).abs();
                Ok(LingotValue::Bool(diff >= 1e-10))
            }
            BinOp::Greater => Ok(LingotValue::Bool(a.to_f64() > b.to_f64())),
            BinOp::Less => Ok(LingotValue::Bool(a.to_f64() < b.to_f64())),
            BinOp::GreaterEqual => Ok(LingotValue::Bool(a.to_f64() >= b.to_f64())),
            BinOp::LessEqual => Ok(LingotValue::Bool(a.to_f64() <= b.to_f64())),
            _ => Err(format!("operator {:?} not supported for Number", op)),
        }
    }

    fn eval_unary(&self, op: &UnaryOp, val: LingotValue) -> Result<LingotValue, String> {
        match (op, val) {
            (UnaryOp::Negate, LingotValue::Number(n)) => {
                Ok(LingotValue::Number(LingotNumber::from_f64(-n.to_f64())))
            }
            (UnaryOp::Not, LingotValue::Bool(b)) => Ok(LingotValue::Bool(!b)),
            (op, val) => Err(format!("operator {:?} not supported for {}", op, type_name(&val))),
        }
    }

    fn call_function(
        &mut self,
        func: LingotValue,
        args: Vec<LingotValue>,
    ) -> Result<LingotValue, String> {
        match func {
            LingotValue::Func { name, params, body } => {
                if args.len() != params.len() {
                    return Err(format!(
                        "{}() expected {} arguments, got {}",
                        name,
                        params.len(),
                        args.len()
                    ));
                }

                self.env.push_scope();
                for (param, arg) in params.iter().zip(args) {
                    self.env.define(&param.name, arg, false, false)?;
                }

                let result = self.execute(&body)?;
                self.env.pop_scope();

                // Auto-unwrap: propagate fail as Err
                if !result.ok {
                    return Err(result.error.unwrap_or_else(|| "unknown error".to_string()));
                }
                Ok(result.value)
            }
            _ => Err(format!("{} is not callable", type_name(&func))),
        }
    }

    /// Try to resolve a builtin function call. Returns None if not a builtin.
    fn try_builtin(
        &mut self,
        name: &str,
        args: &[Expr],
    ) -> Result<Option<LingotValue>, String> {
        match name {
            "__display" | "display" => {
                if args.len() != 1 {
                    return Err("display() takes exactly 1 argument".to_string());
                }
                let val = self.eval_expr(&args[0])?;
                primitives::io_print(&val);
                Ok(Some(LingotValue::Void))
            }
            "shell" => {
                if args.len() != 1 {
                    return Err("shell() takes exactly 1 argument".to_string());
                }
                let val = self.eval_expr(&args[0])?;
                let cmd = match val {
                    LingotValue::Text(s) => s,
                    _ => return Err("shell() argument must be Text".to_string()),
                };
                let result = primitives::process_exec(&cmd)?;
                Ok(Some(result))
            }
            "read" => {
                if args.len() != 1 {
                    return Err("read() takes exactly 1 argument".to_string());
                }
                let val = self.eval_expr(&args[0])?;
                let path = match val {
                    LingotValue::Text(s) => s,
                    _ => return Err("read() argument must be Text".to_string()),
                };
                let result = primitives::fs_read(&path)?;
                Ok(Some(result))
            }
            "delete" => {
                if args.len() != 1 {
                    return Err("delete() takes exactly 1 argument".to_string());
                }
                let val = self.eval_expr(&args[0])?;
                let path = match val {
                    LingotValue::Text(s) => s,
                    _ => return Err("delete() argument must be Text".to_string()),
                };
                let result = primitives::fs_delete(&path)?;
                Ok(Some(result))
            }
            "list" => {
                if args.len() != 1 {
                    return Err("list() takes exactly 1 argument".to_string());
                }
                let val = self.eval_expr(&args[0])?;
                let path = match val {
                    LingotValue::Text(s) => s,
                    _ => return Err("list() argument must be Text".to_string()),
                };
                let result = primitives::fs_list(&path)?;
                Ok(Some(result))
            }
            // Builder starters — return a tagged object
            "move" => {
                if args.len() != 1 {
                    return Err("move() takes exactly 1 argument".to_string());
                }
                let val = self.eval_expr(&args[0])?;
                let path = match val {
                    LingotValue::Text(s) => s,
                    _ => return Err("move() argument must be Text".to_string()),
                };
                Ok(Some(self.make_builder("MoveBuilder", "source", path)))
            }
            "write" => {
                if args.len() != 1 {
                    return Err("write() takes exactly 1 argument".to_string());
                }
                let val = self.eval_expr(&args[0])?;
                let content = match val {
                    LingotValue::Text(s) => s,
                    other => format!("{}", other),
                };
                Ok(Some(self.make_builder("WriteBuilder", "content", content)))
            }
            "rename" => {
                if args.len() != 1 {
                    return Err("rename() takes exactly 1 argument".to_string());
                }
                let val = self.eval_expr(&args[0])?;
                let path = match val {
                    LingotValue::Text(s) => s,
                    _ => return Err("rename() argument must be Text".to_string()),
                };
                Ok(Some(self.make_builder("RenameBuilder", "source", path)))
            }
            "prefix" => {
                if args.len() != 1 {
                    return Err("prefix() takes exactly 1 argument".to_string());
                }
                let val = self.eval_expr(&args[0])?;
                let path = match val {
                    LingotValue::Text(s) => s,
                    _ => return Err("prefix() argument must be Text".to_string()),
                };
                Ok(Some(self.make_builder("PrefixBuilder", "source", path)))
            }
            "suffix" => {
                if args.len() != 1 {
                    return Err("suffix() takes exactly 1 argument".to_string());
                }
                let val = self.eval_expr(&args[0])?;
                let path = match val {
                    LingotValue::Text(s) => s,
                    _ => return Err("suffix() argument must be Text".to_string()),
                };
                Ok(Some(self.make_builder("SuffixBuilder", "source", path)))
            }
            "unzip" => {
                if args.len() != 1 {
                    return Err("unzip() takes exactly 1 argument".to_string());
                }
                let val = self.eval_expr(&args[0])?;
                let path = match val {
                    LingotValue::Text(s) => s,
                    _ => return Err("unzip() argument must be Text".to_string()),
                };
                Ok(Some(self.make_builder("UnzipBuilder", "source", path)))
            }
            _ => Ok(None), // Not a builtin
        }
    }

    fn make_builder(&self, type_name: &str, field: &str, value: String) -> LingotValue {
        let mut fields = std::collections::HashMap::new();
        fields.insert(field.to_string(), LingotValue::Text(value));
        LingotValue::Object {
            type_name: type_name.to_string(),
            fields,
        }
    }

    fn call_method(
        &mut self,
        obj: LingotValue,
        method: &str,
        args: Vec<LingotValue>,
    ) -> Result<LingotValue, String> {
        match obj {
            LingotValue::Object { ref type_name, ref fields } => {
                self.exec_builder_method(type_name, fields, method, &args)
            }
            _ => Err(format!(".{}() is not available on {}", method, type_name(&obj))),
        }
    }

    fn exec_builder_method(
        &self,
        builder_type: &str,
        fields: &std::collections::HashMap<String, LingotValue>,
        method: &str,
        args: &[LingotValue],
    ) -> Result<LingotValue, String> {
        let get_field = |name: &str| -> Result<String, String> {
            match fields.get(name) {
                Some(LingotValue::Text(s)) => Ok(s.clone()),
                _ => Err(format!("builder missing field '{}'", name)),
            }
        };

        let expect_text_arg = |args: &[LingotValue], idx: usize| -> Result<String, String> {
            match args.get(idx) {
                Some(LingotValue::Text(s)) => Ok(s.clone()),
                Some(other) => Ok(format!("{}", other)),
                None => Err(format!(".{}() requires an argument", method)),
            }
        };

        match (builder_type, method) {
            ("MoveBuilder", "to") => {
                let src = get_field("source")?;
                let dest = expect_text_arg(args, 0)?;
                primitives::fs_rename(&src, &dest)
            }
            ("WriteBuilder", "to") => {
                let content = get_field("content")?;
                let path = expect_text_arg(args, 0)?;
                primitives::fs_write(&path, &content)
            }
            ("RenameBuilder", "to") => {
                let src = get_field("source")?;
                let new_name = expect_text_arg(args, 0)?;
                primitives::fs_rename_file(&src, &new_name)
            }
            ("PrefixBuilder", "with") => {
                let src = get_field("source")?;
                let prefix = expect_text_arg(args, 0)?;
                primitives::text_prefix(&src, &prefix)
            }
            ("SuffixBuilder", "with") => {
                let src = get_field("source")?;
                let suffix = expect_text_arg(args, 0)?;
                primitives::text_suffix(&src, &suffix)
            }
            ("UnzipBuilder", "to") => {
                let _src = get_field("source")?;
                let _dest = expect_text_arg(args, 0)?;
                // TODO: implement unzip
                Err("unzip is not yet implemented".to_string())
            }
            _ => Err(format!(".{}() is not available on {}", method, builder_type)),
        }
    }

    fn access_field(&self, obj: LingotValue, field: &str) -> Result<LingotValue, String> {
        match obj {
            LingotValue::Object { fields, .. } => {
                fields.get(field)
                    .cloned()
                    .ok_or_else(|| format!("field '{}' not found", field))
            }
            _ => Err(format!("cannot access field '{}' on {}", field, type_name(&obj))),
        }
    }

    fn eval_range(&self, start: LingotValue, end: LingotValue) -> Result<LingotValue, String> {
        match (&start, &end) {
            (LingotValue::Number(s), LingotValue::Number(e)) => {
                let s = s.to_i64();
                let e = e.to_i64();
                let items: Vec<LingotValue> = (s..e)
                    .map(|i| LingotValue::Number(LingotNumber::Int(i)))
                    .collect();
                Ok(LingotValue::List(items))
            }
            _ => Err("range requires Number values".to_string()),
        }
    }

    fn to_iterable(&self, val: &LingotValue) -> Result<Vec<LingotValue>, String> {
        match val {
            LingotValue::List(items) => Ok(items.clone()),
            _ => Err(format!("cannot iterate over {}", type_name(val))),
        }
    }

    fn is_truthy(&self, val: &LingotValue) -> bool {
        match val {
            LingotValue::Bool(b) => *b,
            LingotValue::Number(n) => n.to_f64() != 0.0,
            LingotValue::Text(s) => !s.is_empty(),
            LingotValue::Void => false,
            LingotValue::List(items) => !items.is_empty(),
            _ => true,
        }
    }
}

fn type_name(val: &LingotValue) -> &str {
    match val {
        LingotValue::Text(_) => "Text",
        LingotValue::Number(_) => "Number",
        LingotValue::Bool(_) => "Bool",
        LingotValue::Void => "Void",
        LingotValue::List(_) => "List",
        LingotValue::Map(_) => "Map",
        LingotValue::Object { .. } => "Object",
        LingotValue::Func { .. } => "Func",
    }
}
