use crate::Expr;
use crate::token::Token;
use logos::Logos;
use std::collections::VecDeque;

#[derive(Debug)]
pub(crate) enum ParsedExpr {
    And,
    Or,
    Not,
    LParen,
    RParen,
    Expr(Expr),
}

#[derive(Debug)]
pub(crate) struct Parser {
    parsed: VecDeque<ParsedExpr>,
}

impl Parser {
    pub(crate) fn new(s: &str) -> crate::Result<Self> {
        let parsed = tokenize(s)?;

        Ok(Self { parsed })
    }

    pub(crate) fn parse(&mut self) -> crate::Result<ParsedExpr> {
        loop {
            let Some((start, end)) = self.find_innermost_subexpression()? else {
                break;
            };

            let mut after = self.parsed.split_off(end);
            after.pop_front();

            let mut expr = self.parsed.split_off(start);
            expr.pop_front();

            let mut subp = Parser::subparser(expr);
            let v = subp.parse_plain()?;

            self.parsed.push_back(v);
            self.parsed.extend(after);
        }

        self.parse_plain()
    }

    fn subparser(parsed: VecDeque<ParsedExpr>) -> Self {
        Self { parsed }
    }

    fn parse_plain(&mut self) -> crate::Result<ParsedExpr> {
        self.apply_except()?;
        self.insert_ands()?;
        self.apply_and()?;
        self.apply_or()?;

        if self.parsed.len() == 1 {
            let expr = self.parsed.remove(0).unwrap();
            if matches!(expr, ParsedExpr::Expr(_)) {
                Ok(expr)
            } else {
                crate::err!("syntax error; expression parsed to {:?}", expr)
            }
        } else {
            crate::err!("syntax error; expression parsed to {:?}", self.parsed)
        }
    }

    fn find_innermost_subexpression(&mut self) -> crate::Result<Option<(usize, usize)>> {
        let mut start: Option<usize> = None;
        for (idx, node) in self.parsed.iter().enumerate() {
            match node {
                ParsedExpr::LParen => {
                    start = Some(idx);
                }
                ParsedExpr::RParen => {
                    if let Some(start) = start {
                        return Ok(Some((start, idx)));
                    } else {
                        return crate::err!("too many ')'");
                    }
                }
                _ => (),
            }
        }

        if start.is_some() {
            crate::err!("too many '('")
        } else {
            Ok(None)
        }
    }

    fn insert_ands(&mut self) -> crate::Result<()> {
        let mut prev_is_expr = false;

        let mut tmp = VecDeque::<ParsedExpr>::new();
        for expr in self.parsed.drain(..) {
            let curr_is_expr = matches!(expr, ParsedExpr::Expr(_));
            if prev_is_expr && curr_is_expr {
                tmp.push_back(ParsedExpr::And);
            }

            tmp.push_back(expr);

            prev_is_expr = curr_is_expr;
        }

        self.parsed = tmp;

        Ok(())
    }

    fn apply_except(&mut self) -> crate::Result<()> {
        while let Some(idx) = self
            .parsed
            .iter()
            .rposition(|item| matches!(item, ParsedExpr::Not))
        {
            let arg = self.drop_next(idx)?;
            let arg: Expr = arg.try_into()?;
            let expr = Expr::Not(Box::new(arg));

            self.parsed[idx] = ParsedExpr::Expr(expr);
        }

        Ok(())
    }

    fn drop_prev(&mut self, idx: usize) -> crate::Result<ParsedExpr> {
        if idx == 0 {
            return crate::err!("no expression on the right");
        }

        match self.parsed.remove(idx - 1) {
            None => crate::err!("no expression on the right"),
            Some(expr) => Ok(expr),
        }
    }

    fn drop_next(&mut self, idx: usize) -> crate::Result<ParsedExpr> {
        match self.parsed.remove(idx + 1) {
            None => crate::err!("no expression on the left"),
            Some(expr) => Ok(expr),
        }
    }

    fn apply_and(&mut self) -> crate::Result<()> {
        while let Some(idx) = self
            .parsed
            .iter()
            .position(|item| matches!(item, ParsedExpr::And))
        {
            let rhs = self.drop_next(idx)?;
            let lhs = self.drop_prev(idx)?;
            let idx = idx - 1;

            if let ParsedExpr::Expr(Expr::And(mut lhs)) = lhs {
                let rhs: Expr = rhs.try_into()?;

                lhs.push(rhs);

                self.parsed[idx] = ParsedExpr::Expr(Expr::And(lhs));
            } else {
                let lhs: Expr = lhs.try_into()?;
                let rhs: Expr = rhs.try_into()?;

                self.parsed[idx] = ParsedExpr::Expr(Expr::And(vec![lhs, rhs]));
            }
        }

        Ok(())
    }

    fn apply_or(&mut self) -> crate::Result<()> {
        while let Some(idx) = self
            .parsed
            .iter()
            .position(|item| matches!(item, ParsedExpr::Or))
        {
            let rhs = self.drop_next(idx)?;
            let lhs = self.drop_prev(idx)?;
            let idx = idx - 1;

            if let ParsedExpr::Expr(Expr::Or(mut lhs)) = lhs {
                let rhs: Expr = rhs.try_into()?;

                lhs.push(rhs);

                self.parsed[idx] = ParsedExpr::Expr(Expr::Or(lhs));
            } else {
                let lhs: Expr = lhs.try_into()?;
                let rhs: Expr = rhs.try_into()?;

                self.parsed[idx] = ParsedExpr::Expr(Expr::Or(vec![lhs, rhs]));
            }
        }

        Ok(())
    }
}

impl TryFrom<ParsedExpr> for Expr {
    type Error = String;

    fn try_from(p: ParsedExpr) -> Result<Self, Self::Error> {
        match p {
            ParsedExpr::Expr(expr) => Ok(expr),
            _ => crate::err!("cannot cast '{p:?}' into an expression"),
        }
    }
}

fn tokenize(s: &str) -> crate::Result<VecDeque<ParsedExpr>> {
    let mut result = VecDeque::<ParsedExpr>::new();

    for (item, _span) in Token::lexer(s).spanned() {
        match item {
            Ok(token) => {
                let p = match token {
                    Token::Except => ParsedExpr::Not,
                    Token::Or => ParsedExpr::Or,
                    Token::And => ParsedExpr::And,
                    Token::LParen => ParsedExpr::LParen,
                    Token::RParen => ParsedExpr::RParen,
                    Token::Substring(s) => ParsedExpr::Expr(Expr::Substring(s)),
                    Token::Prefix(s) => ParsedExpr::Expr(Expr::Prefix(s)),
                    Token::ExactString(s) => ParsedExpr::Expr(Expr::ExactString(s)),
                };
                result.push_back(p);
            }
            Err(err) => {
                return crate::err!("{err:?}");
            }
        }
    }

    Ok(result)
}
