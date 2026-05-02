use crate::{
    ast::{Expr, OpBin, OpUn, Stmt},
    error::PiolaError,
    lexer::token::{Span, Token, TokenKind},
};

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    // ── Token inspection ────────────────────────────────────────────────────

    fn peek(&self) -> &TokenKind {
        &self.tokens[self.pos].kind
    }

    fn peek_span(&self) -> &Span {
        &self.tokens[self.pos].span
    }

    fn peek_next(&self) -> &TokenKind {
        if self.pos + 1 < self.tokens.len() {
            &self.tokens[self.pos + 1].kind
        } else {
            &TokenKind::EOF
        }
    }

    fn advance(&mut self) -> &Token {
        let tok = &self.tokens[self.pos];
        if self.pos + 1 < self.tokens.len() {
            self.pos += 1;
        }
        tok
    }

    fn check(&self, kind: &TokenKind) -> bool {
        self.peek() == kind
    }

    fn consume(&mut self, kind: &TokenKind) -> Result<&Token, PiolaError> {
        if self.peek() == kind {
            Ok(self.advance())
        } else {
            Err(PiolaError::Sintaxis(format!(
                "Esperaba {:?} pero encontré {:?} en posición {}.",
                kind,
                self.peek(),
                self.peek_span().start,
            )))
        }
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek(), TokenKind::EOF)
    }

    // ── Top-level ────────────────────────────────────────────────────────────

    pub fn parsear(&mut self) -> Result<Vec<Stmt>, PiolaError> {
        let mut stmts = Vec::new();
        while !self.is_at_end() {
            stmts.push(self.parse_stmt()?);
        }
        Ok(stmts)
    }

    // ── Statements ───────────────────────────────────────────────────────────

    fn parse_stmt(&mut self) -> Result<Stmt, PiolaError> {
        match self.peek() {
            TokenKind::Wea => self.parse_decl_wea(),
            TokenKind::Duro => self.parse_decl_duro(),
            TokenKind::Pega => self.parse_decl_pega(),
            TokenKind::Cachai => self.parse_cachai(),
            TokenKind::Mientras => self.parse_mientras(),
            TokenKind::Para => self.parse_para(),
            TokenKind::Ojo => self.parse_ojo(),
            _ => self.parse_expr_stmt(),
        }
    }

    fn parse_decl_wea(&mut self) -> Result<Stmt, PiolaError> {
        self.advance(); // consume 'wea'
        let nombre = self.expect_ident()?;
        self.consume(&TokenKind::Asignar)?;
        let valor = self.parse_expr()?;
        Ok(Stmt::DeclWea { nombre, valor, es_duro: false })
    }

    fn parse_decl_duro(&mut self) -> Result<Stmt, PiolaError> {
        self.advance(); // consume 'duro'
        let nombre = self.expect_ident()?;
        self.consume(&TokenKind::Asignar)?;
        let valor = self.parse_expr()?;
        Ok(Stmt::DeclWea { nombre, valor, es_duro: true })
    }

    fn parse_decl_pega(&mut self) -> Result<Stmt, PiolaError> {
        self.advance(); // consume 'pega'
        let nombre = self.expect_ident()?;
        self.consume(&TokenKind::LParen)?;
        let params = self.parse_params()?;
        self.consume(&TokenKind::RParen)?;
        let cuerpo = self.parse_block()?;
        Ok(Stmt::DeclPega { nombre, params, cuerpo })
    }

    fn parse_params(&mut self) -> Result<Vec<String>, PiolaError> {
        let mut params = Vec::new();
        if self.check(&TokenKind::RParen) {
            return Ok(params);
        }
        params.push(self.expect_ident()?);
        while self.check(&TokenKind::Coma) {
            self.advance();
            params.push(self.expect_ident()?);
        }
        Ok(params)
    }

    fn parse_cachai(&mut self) -> Result<Stmt, PiolaError> {
        self.advance(); // consume 'cachai'
        self.consume(&TokenKind::LParen)?;
        let cond = self.parse_expr()?;
        self.consume(&TokenKind::RParen)?;
        let entonces = self.parse_block()?;

        // Check for 'si no' (two-token else)
        let si_no = if self.check(&TokenKind::Si) && self.peek_next() == &TokenKind::No {
            self.advance(); // consume 'si'
            self.advance(); // consume 'no'
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(Stmt::Cachai { cond, entonces, si_no })
    }

    fn parse_mientras(&mut self) -> Result<Stmt, PiolaError> {
        self.advance(); // consume 'mientras'
        self.consume(&TokenKind::LParen)?;
        let cond = self.parse_expr()?;
        self.consume(&TokenKind::RParen)?;
        let cuerpo = self.parse_block()?;
        Ok(Stmt::Mientras { cond, cuerpo })
    }

    fn parse_para(&mut self) -> Result<Stmt, PiolaError> {
        self.advance(); // consume 'para'
        self.consume(&TokenKind::LParen)?;
        let var = self.expect_ident()?;
        self.consume(&TokenKind::En)?;
        let iterable = self.parse_expr()?;
        self.consume(&TokenKind::RParen)?;
        let cuerpo = self.parse_block()?;
        Ok(Stmt::Para { var, iterable, cuerpo })
    }

    fn parse_ojo(&mut self) -> Result<Stmt, PiolaError> {
        self.advance(); // consume 'ojo'
        let cuerpo = self.parse_block()?;
        self.consume(&TokenKind::Cago)?;
        self.consume(&TokenKind::LParen)?;
        let error_var = self.expect_ident()?;
        self.consume(&TokenKind::RParen)?;
        let manejo = self.parse_block()?;
        Ok(Stmt::Ojo { cuerpo, error_var, manejo })
    }

    fn parse_expr_stmt(&mut self) -> Result<Stmt, PiolaError> {
        let expr = self.parse_expr()?;
        Ok(Stmt::Expresion(expr))
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, PiolaError> {
        self.consume(&TokenKind::LLave)?;
        let mut stmts = Vec::new();
        while !self.check(&TokenKind::RLlave) && !self.is_at_end() {
            stmts.push(self.parse_stmt()?);
        }
        self.consume(&TokenKind::RLlave)?;
        Ok(stmts)
    }

    // ── Expressions (Pratt) ──────────────────────────────────────────────────

    fn parse_expr(&mut self) -> Result<Expr, PiolaError> {
        self.parse_pratt(0)
    }

    fn parse_pratt(&mut self, min_bp: u8) -> Result<Expr, PiolaError> {
        let mut lhs = self.parse_unary()?;

        loop {
            let op = match self.peek() {
                TokenKind::O => OpBin::O,
                TokenKind::Y => OpBin::Y,
                TokenKind::IgualIgual => OpBin::Eq,
                TokenKind::BangIgual => OpBin::Neq,
                TokenKind::Menor => OpBin::Lt,
                TokenKind::Mayor => OpBin::Gt,
                TokenKind::MenorIgual => OpBin::Lte,
                TokenKind::MayorIgual => OpBin::Gte,
                TokenKind::Mas => OpBin::Suma,
                TokenKind::Menos => OpBin::Resta,
                TokenKind::Estrella => OpBin::Mul,
                TokenKind::Diagonal => OpBin::Div,
                TokenKind::Modulo => OpBin::Mod,
                _ => break,
            };

            let (left_bp, right_bp) = infix_binding_power(&op);
            if left_bp < min_bp {
                break;
            }

            let span_start = self.peek_span().start;
            self.advance(); // consume operator
            let rhs = self.parse_pratt(right_bp)?;
            let span = Span::new(span_start, self.peek_span().start);

            lhs = Expr::Binario {
                izq: Box::new(lhs),
                op,
                der: Box::new(rhs),
                span,
            };
        }

        Ok(lhs)
    }

    fn parse_unary(&mut self) -> Result<Expr, PiolaError> {
        match self.peek() {
            TokenKind::No => {
                let span_start = self.peek_span().start;
                self.advance();
                let expr = self.parse_unary()?;
                let span = Span::new(span_start, self.peek_span().start);
                Ok(Expr::Unario { op: OpUn::No, expr: Box::new(expr), span })
            }
            TokenKind::Menos => {
                let span_start = self.peek_span().start;
                self.advance();
                let expr = self.parse_unary()?;
                let span = Span::new(span_start, self.peek_span().start);
                Ok(Expr::Unario { op: OpUn::Neg, expr: Box::new(expr), span })
            }
            _ => self.parse_postfix(),
        }
    }

    fn parse_postfix(&mut self) -> Result<Expr, PiolaError> {
        let mut expr = self.parse_primary()?;

        loop {
            match self.peek() {
                TokenKind::LParen => {
                    let span_start = self.peek_span().start;
                    self.advance(); // consume '('
                    let args = self.parse_args()?;
                    self.consume(&TokenKind::RParen)?;
                    let span = Span::new(span_start, self.peek_span().start);
                    expr = Expr::Llamada { callee: Box::new(expr), args, span };
                }
                TokenKind::LCorchete => {
                    let span_start = self.peek_span().start;
                    self.advance(); // consume '['
                    let indice = self.parse_expr()?;
                    self.consume(&TokenKind::RCorchete)?;
                    let span = Span::new(span_start, self.peek_span().start);
                    expr = Expr::Indice { objeto: Box::new(expr), indice: Box::new(indice), span };
                }
                _ => break,
            }
        }

        Ok(expr)
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>, PiolaError> {
        let mut args = Vec::new();
        if self.check(&TokenKind::RParen) {
            return Ok(args);
        }
        args.push(self.parse_expr()?);
        while self.check(&TokenKind::Coma) {
            self.advance();
            args.push(self.parse_expr()?);
        }
        Ok(args)
    }

    fn parse_primary(&mut self) -> Result<Expr, PiolaError> {
        match self.peek().clone() {
            TokenKind::Numero(n) => {
                self.advance();
                Ok(Expr::Numero(n))
            }
            TokenKind::Texto(s) => {
                self.advance();
                Ok(Expr::Texto(s))
            }
            TokenKind::Verdad => {
                self.advance();
                Ok(Expr::Booleano(true))
            }
            TokenKind::Falso => {
                self.advance();
                Ok(Expr::Booleano(false))
            }
            TokenKind::Nada => {
                self.advance();
                Ok(Expr::Nada)
            }
            TokenKind::Ident(nombre) => {
                let span = self.peek_span().clone();
                self.advance();

                // Check for bare assignment: ident = expr (not ==)
                if self.check(&TokenKind::Asignar) {
                    self.advance(); // consume '='
                    let valor = self.parse_expr()?;
                    let end = self.peek_span().start;
                    return Ok(Expr::Asignacion {
                        nombre,
                        valor: Box::new(valor),
                        span: Span::new(span.start, end),
                    });
                }

                Ok(Expr::Ident(nombre, span))
            }
            TokenKind::LParen => {
                self.advance();
                let expr = self.parse_expr()?;
                self.consume(&TokenKind::RParen)?;
                Ok(expr)
            }
            TokenKind::LCorchete => {
                let span_start = self.peek_span().start;
                self.advance();
                let mut items = Vec::new();
                if !self.check(&TokenKind::RCorchete) {
                    items.push(self.parse_expr()?);
                    while self.check(&TokenKind::Coma) {
                        self.advance();
                        if self.check(&TokenKind::RCorchete) {
                            break;
                        }
                        items.push(self.parse_expr()?);
                    }
                }
                self.consume(&TokenKind::RCorchete)?;
                let span = Span::new(span_start, self.peek_span().start);
                Ok(Expr::Lista(items, span))
            }
            TokenKind::LLave => {
                let span_start = self.peek_span().start;
                self.advance();
                let mut pairs = Vec::new();
                if !self.check(&TokenKind::RLlave) {
                    let key = self.parse_expr()?;
                    self.consume(&TokenKind::Colon)?;
                    let val = self.parse_expr()?;
                    pairs.push((key, val));
                    while self.check(&TokenKind::Coma) {
                        self.advance();
                        if self.check(&TokenKind::RLlave) {
                            break;
                        }
                        let key = self.parse_expr()?;
                        self.consume(&TokenKind::Colon)?;
                        let val = self.parse_expr()?;
                        pairs.push((key, val));
                    }
                }
                self.consume(&TokenKind::RLlave)?;
                let span = Span::new(span_start, self.peek_span().start);
                Ok(Expr::Mapa(pairs, span))
            }
            other => Err(PiolaError::Sintaxis(format!(
                "Expresión inesperada: {:?} en posición {}.",
                other,
                self.peek_span().start,
            ))),
        }
    }

    fn expect_ident(&mut self) -> Result<String, PiolaError> {
        match self.peek().clone() {
            TokenKind::Ident(s) => {
                self.advance();
                Ok(s)
            }
            other => Err(PiolaError::Sintaxis(format!(
                "Esperaba un identificador pero encontré {:?} en posición {}.",
                other,
                self.peek_span().start,
            ))),
        }
    }
}

// Pratt binding powers: (left_bp, right_bp)
fn infix_binding_power(op: &OpBin) -> (u8, u8) {
    match op {
        OpBin::O => (1, 2),
        OpBin::Y => (3, 4),
        OpBin::Eq | OpBin::Neq => (5, 6),
        OpBin::Lt | OpBin::Gt | OpBin::Lte | OpBin::Gte => (7, 8),
        OpBin::Suma | OpBin::Resta => (9, 10),
        OpBin::Mul | OpBin::Div | OpBin::Mod => (11, 12),
    }
}

pub fn parsear(tokens: Vec<Token>) -> Result<Vec<Stmt>, PiolaError> {
    Parser::new(tokens).parsear()
}