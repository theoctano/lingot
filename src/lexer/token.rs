#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    /// Number literal. bool = has decimal point (force float)
    NumberLit(f64, bool),
    TextLit(String),
    BoolLit(bool),
    /// Interpolated string: alternating text parts and expression source strings
    /// e.g. "Hello {name}, you are {age}" → [("Hello ", "name"), (", you are ", "age")]
    /// Final trailing text is stored as last element with empty expr
    InterpolatedText(Vec<(String, String)>),

    // Identifiers
    Identifier(String),

    // Keywords
    Let,
    Dyn,
    Pub,
    If,
    Else,
    While,
    Repeat,
    For,
    In,
    Try,
    Catch,
    Fail,
    Return,
    Load,
    From,
    ObjectKw,

    // Operator keywords (aliases)
    And,
    Or,
    Not,
    Is,
    Greater,
    Lesser,
    Than,
    Equal,

    // Symbols
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Assign,
    EqualEqual,
    BangEqual,
    GreaterThan,
    LessThan,
    GreaterEqual,
    LessEqual,
    AmpAmp,
    PipePipe,
    Bang,

    // Delimiters
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    LeftBracket,
    RightBracket,
    Comma,
    Dot,
    DotDot,
    Colon,
    Semicolon,

    // Special
    Eof,
}

impl Token {
    pub fn from_keyword(word: &str) -> Option<Token> {
        match word {
            "let" => Some(Token::Let),
            "dyn" => Some(Token::Dyn),
            "pub" => Some(Token::Pub),
            "if" => Some(Token::If),
            "else" => Some(Token::Else),
            "while" => Some(Token::While),
            "repeat" => Some(Token::Repeat),
            "for" => Some(Token::For),
            "in" => Some(Token::In),
            "try" => Some(Token::Try),
            "catch" => Some(Token::Catch),
            "fail" => Some(Token::Fail),
            "return" => Some(Token::Return),
            "load" => Some(Token::Load),
            "from" => Some(Token::From),
            "Object" => Some(Token::ObjectKw),
            "true" => Some(Token::BoolLit(true)),
            "false" => Some(Token::BoolLit(false)),
            "and" => Some(Token::And),
            "or" => Some(Token::Or),
            "not" => Some(Token::Not),
            "is" => Some(Token::Is),
            "greater" => Some(Token::Greater),
            "lesser" => Some(Token::Lesser),
            "than" => Some(Token::Than),
            "equal" => Some(Token::Equal),
            _ => None,
        }
    }
}
