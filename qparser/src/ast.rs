/// AST for Q expressions: Phase 0 atoms & basic arithmetic.
#[derive(Debug, PartialEq)]
pub enum Expr {
    /// Integer literal
    Int(i64),
    /// Floating-point literal
    Float(f64),
    /// Addition
    Add(Box<Expr>, Box<Expr>),
    /// Subtraction
    Sub(Box<Expr>, Box<Expr>),
    /// Multiplication
    Mul(Box<Expr>, Box<Expr>),
    /// Division
    Div(Box<Expr>, Box<Expr>),
}
// Pretty-print atomic AST nodes
impl std::fmt::Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Int(i) => write!(f, "{}", i),
            Expr::Float(x) => write!(f, "{}", x),
            Expr::Add(l, r) => write!(f, "({} + {})", l, r),
            Expr::Sub(l, r) => write!(f, "({} - {})", l, r),
            Expr::Mul(l, r) => write!(f, "({} * {})", l, r),
            Expr::Div(l, r) => write!(f, "({} / {})", l, r),
        }
    }
}

impl Expr {
    /// Evaluate the AST into a new atomic Expr.
    pub fn eval(&self) -> Result<Expr, String> {
        match self {
            Expr::Int(i) => Ok(Expr::Int(*i)),
            Expr::Float(f) => Ok(Expr::Float(*f)),
            Expr::Add(l, r) => {
                let left = l.eval()?;
                let right = r.eval()?;
                match (left, right) {
                    (Expr::Int(a), Expr::Int(b)) => Ok(Expr::Int(a + b)),
                    (Expr::Int(a), Expr::Float(b)) => Ok(Expr::Float(a as f64 + b)),
                    (Expr::Float(a), Expr::Int(b)) => Ok(Expr::Float(a + b as f64)),
                    (Expr::Float(a), Expr::Float(b)) => Ok(Expr::Float(a + b)),
                    _ => Err("Type error in addition".into()),
                }
            }
            Expr::Sub(l, r) => {
                let left = l.eval()?;
                let right = r.eval()?;
                match (left, right) {
                    (Expr::Int(a), Expr::Int(b)) => Ok(Expr::Int(a - b)),
                    (Expr::Int(a), Expr::Float(b)) => Ok(Expr::Float(a as f64 - b)),
                    (Expr::Float(a), Expr::Int(b)) => Ok(Expr::Float(a - b as f64)),
                    (Expr::Float(a), Expr::Float(b)) => Ok(Expr::Float(a - b)),
                    _ => Err("Type error in subtraction".into()),
                }
            }
            Expr::Mul(l, r) => {
                let left = l.eval()?;
                let right = r.eval()?;
                match (left, right) {
                    (Expr::Int(a), Expr::Int(b)) => Ok(Expr::Int(a * b)),
                    (Expr::Int(a), Expr::Float(b)) => Ok(Expr::Float(a as f64 * b)),
                    (Expr::Float(a), Expr::Int(b)) => Ok(Expr::Float(a * b as f64)),
                    (Expr::Float(a), Expr::Float(b)) => Ok(Expr::Float(a * b)),
                    _ => Err("Type error in multiplication".into()),
                }
            }
            Expr::Div(l, r) => {
                let left = l.eval()?;
                let right = r.eval()?;
                match &right {
                    Expr::Int(b) if *b == 0 => return Err("Division by zero".into()),
                    Expr::Float(b) if *b == 0.0 => return Err("Division by zero".into()),
                    _ => (),
                };
                match (left, right) {
                    (Expr::Int(a), Expr::Int(b)) => Ok(Expr::Float(a as f64 / b as f64)),
                    (Expr::Int(a), Expr::Float(b)) => Ok(Expr::Float(a as f64 / b)),
                    (Expr::Float(a), Expr::Int(b)) => Ok(Expr::Float(a / b as f64)),
                    (Expr::Float(a), Expr::Float(b)) => Ok(Expr::Float(a / b)),
                    _ => Err("Type error in division".into()),
                }
            }
        }
    }
}
