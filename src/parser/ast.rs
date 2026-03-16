#[derive(Debug, Clone)]
pub enum Expr {
    /// Number literal. bool = is_float (written with decimal point)
    NumberLit(f64, bool),
    TextLit(String),
    BoolLit(bool),
    Identifier(String),
    Binary {
        left: Box<Expr>,
        op: BinOp,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    MethodCall {
        object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    FieldAccess {
        object: Box<Expr>,
        field: String,
    },
    ListLit(Vec<Expr>),
    MapLit(Vec<(Expr, Expr)>),
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
    },
    Interpolation(Vec<InterpolationPart>),
    Lambda {
        params: Vec<Param>,
        body: Vec<Stmt>,
    },
}

#[derive(Debug, Clone)]
pub enum InterpolationPart {
    Text(String),
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    EqualEqual,
    NotEqual,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    And,
    Or,
}

#[derive(Debug, Clone)]
pub enum UnaryOp {
    Negate,
    Not,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub type_annotation: Option<String>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    Let {
        name: String,
        is_dyn: bool,
        is_pub: bool,
        type_annotation: Option<String>,
        value: Expr,
    },
    FuncDecl {
        name: String,
        is_dyn: bool,
        is_pub: bool,
        params: Vec<Param>,
        body: Vec<Stmt>,
    },
    Assign {
        name: String,
        value: Expr,
    },
    ExprStmt(Expr),
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    RepeatWhile {
        body: Vec<Stmt>,
        condition: Expr,
    },
    RepeatFor {
        var_name: String,
        iterable: Expr,
        body: Vec<Stmt>,
    },
    TryCatch {
        try_body: Vec<Stmt>,
        error_name: String,
        catch_body: Vec<Stmt>,
    },
    Return(Expr),
    Fail(Expr),
    Load {
        items: Vec<String>, // ["*"] for wildcard
        path: String,
    },
}
