use crate::parser::ParsedExpr;
use crate::parser::Parser;
use std::fmt::Display;
use std::fmt::Formatter;

#[derive(Debug, Clone)]
pub enum Expr {
    ExactString(String),
    Prefix(String),
    Substring(String),
    And(Vec<Self>),
    Or(Vec<Self>),
    Not(Box<Self>),
}

impl Expr {
    pub fn new(s: &str) -> crate::Result<Self> {
        let mut p = Parser::new(s)?;
        let expr = p.parse()?;

        if let ParsedExpr::Expr(expr) = expr {
            Ok(expr)
        } else {
            crate::err!("parsed to {expr:?}")
        }
    }

    pub fn matches(&self, parts: &[String]) -> bool {
        match self {
            Self::ExactString(s) => parts.iter().any(|string| string == s),
            Self::Prefix(s) => parts.iter().any(|string| string.starts_with(s)),
            Self::Substring(s) => parts.iter().any(|string| string.contains(s)),
            Self::Not(expr) => !expr.matches(parts),
            Self::And(expressions) => expressions.iter().all(|expr| expr.matches(parts)),
            Self::Or(expressions) => expressions.iter().any(|expr| expr.matches(parts)),
        }
    }

    pub fn matches_chain(&self, parts1: &[String], parts2: &[String]) -> bool {
        match self {
            Self::ExactString(s) => parts1.iter().chain(parts2.iter()).any(|string| string == s),
            Self::Prefix(s) => parts1
                .iter()
                .chain(parts2.iter())
                .any(|string| string.starts_with(s)),
            Self::Substring(s) => parts1
                .iter()
                .chain(parts2.iter())
                .any(|string| string.contains(s)),
            Self::Not(expr) => !expr.matches_chain(parts1, parts2),
            Self::And(expressions) => expressions
                .iter()
                .all(|expr| expr.matches_chain(parts1, parts2)),
            Self::Or(expressions) => expressions
                .iter()
                .any(|expr| expr.matches_chain(parts1, parts2)),
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        match self {
            Self::Or(v) => {
                write!(f, "(")?;
                for (i, expr) in v.iter().enumerate() {
                    if i > 0 {
                        write!(f, " | ")?;
                    }

                    write!(f, "{expr}")?;
                }
                write!(f, ")")?;
            }
            Self::And(v) => {
                write!(f, "(")?;
                for (i, expr) in v.iter().enumerate() {
                    if i > 0 {
                        write!(f, " & ")?;
                    }

                    write!(f, "{expr}")?;
                }
                write!(f, ")")?;
            }
            Self::Not(v) => {
                write!(f, "-{v}")?;
            }
            Self::Substring(s) => {
                write!(f, "{s}")?;
            }
            Self::ExactString(s) => {
                write!(f, "'{s}'")?;
            }
            Self::Prefix(s) => {
                write!(f, "{s}*")?;
            }
        }

        Ok(())
    }
}
