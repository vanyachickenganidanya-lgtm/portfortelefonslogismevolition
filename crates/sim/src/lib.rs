//! Симулятор логики + крошечный интерпретатор Funo (подмножество funo-studio).

use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub type Bit = u8;

#[derive(Clone, Debug)]
pub enum Expr {
    Num(i64),
    Var(String),
    Eq(Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Call(String, Vec<Expr>),
}

#[derive(Clone, Debug)]
pub struct Fun {
    pub name: String,
    pub params: Vec<String>,
    pub body: Expr,
}

#[derive(Clone, Debug, Default)]
pub struct Program {
    pub funs: HashMap<String, Fun>,
}

pub fn parse_funo(src: &str) -> Result<Program, String> {
    let mut p = Parser {
        toks: tokenize(src),
        i: 0,
    };
    p.parse_program()
}

struct Parser {
    toks: Vec<String>,
    i: usize,
}

impl Parser {
    fn peek(&self) -> Option<&str> {
        self.toks.get(self.i).map(|s| s.as_str())
    }
    fn eat(&mut self) -> Result<String, String> {
        let t = self.peek().ok_or("unexpected eof")?.to_string();
        self.i += 1;
        Ok(t)
    }
    fn expect(&mut self, w: &str) -> Result<(), String> {
        let t = self.eat()?;
        if t != w {
            return Err(format!("expected {w}, got {t}"));
        }
        Ok(())
    }
    fn parse_program(&mut self) -> Result<Program, String> {
        let mut prog = Program::default();
        while self.peek().is_some() {
            self.expect("fun")?;
            let name = self.eat()?;
            self.expect("(")?;
            let mut params = Vec::new();
            if self.peek() != Some(")") {
                loop {
                    params.push(self.eat()?);
                    if self.peek() == Some(",") {
                        self.eat()?;
                        continue;
                    }
                    break;
                }
            }
            self.expect(")")?;
            let body = if self.peek() == Some("=") {
                self.eat()?;
                self.parse_expr()?
            } else {
                self.expect("{")?;
                let mut last = Expr::Num(0);
                while self.peek() != Some("}") {
                    last = self.parse_stmt()?;
                }
                self.expect("}")?;
                last
            };
            prog.funs.insert(
                name.clone(),
                Fun {
                    name,
                    params,
                    body,
                },
            );
        }
        Ok(prog)
    }
    fn parse_stmt(&mut self) -> Result<Expr, String> {
        if self.peek() == Some("return") {
            self.eat()?;
            self.expect("(")?;
            let e = self.parse_expr()?;
            self.expect(")")?;
            return Ok(e);
        }
        if self.peek() == Some("if") {
            return self.parse_if();
        }
        self.parse_expr()
    }
    fn parse_if(&mut self) -> Result<Expr, String> {
        self.expect("if")?;
        let c = self.parse_expr()?;
        self.expect("then")?;
        let t = if self.peek() == Some("if") {
            self.parse_if()?
        } else if self.peek() == Some("return") {
            self.parse_stmt()?
        } else {
            self.parse_expr()?
        };
        let e = if self.peek() == Some("else") {
            self.eat()?;
            if self.peek() == Some("if") {
                self.parse_if()?
            } else if self.peek() == Some("return") {
                self.parse_stmt()?
            } else {
                self.parse_expr()?
            }
        } else {
            Expr::Num(0)
        };
        Ok(Expr::If(Box::new(c), Box::new(t), Box::new(e)))
    }
    fn parse_expr(&mut self) -> Result<Expr, String> {
        let left = self.parse_atom()?;
        if self.peek() == Some("==") {
            self.eat()?;
            let r = self.parse_atom()?;
            return Ok(Expr::Eq(Box::new(left), Box::new(r)));
        }
        Ok(left)
    }
    fn parse_atom(&mut self) -> Result<Expr, String> {
        let t = self.eat()?;
        if t.chars().all(|c| c.is_ascii_digit() || c == '-') {
            return Ok(Expr::Num(t.parse().unwrap_or(0)));
        }
        if self.peek() == Some("(") {
            self.eat()?;
            let mut args = Vec::new();
            if self.peek() != Some(")") {
                loop {
                    args.push(self.parse_expr()?);
                    if self.peek() == Some(",") {
                        self.eat()?;
                        continue;
                    }
                    break;
                }
            }
            self.expect(")")?;
            return Ok(Expr::Call(t, args));
        }
        Ok(Expr::Var(t))
    }
}

fn tokenize(src: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut i = 0;
    let b = src.as_bytes();
    while i < b.len() {
        if b[i] == b'#' {
            while i < b.len() && b[i] != b'\n' {
                i += 1;
            }
            continue;
        }
        if b[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        if i + 1 < b.len() && &src[i..i + 2] == "==" {
            out.push("==".into());
            i += 2;
            continue;
        }
        let c = b[i] as char;
        if "(){},=".contains(c) {
            out.push(c.to_string());
            i += 1;
            continue;
        }
        let s = i;
        while i < b.len() {
            let ch = b[i] as char;
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                i += 1;
            } else {
                break;
            }
        }
        if s < i {
            out.push(src[s..i].to_string());
        } else {
            i += 1;
        }
    }
    out
}

pub fn eval_fun(prog: &Program, name: &str, args: &[i64]) -> Result<i64, String> {
    let f = prog.funs.get(name).ok_or_else(|| format!("no fun {name}"))?;
    let mut env = HashMap::new();
    for (p, a) in f.params.iter().zip(args.iter()) {
        env.insert(p.clone(), *a);
    }
    eval_expr(prog, &f.body, &env)
}

fn eval_expr(prog: &Program, e: &Expr, env: &HashMap<String, i64>) -> Result<i64, String> {
    match e {
        Expr::Num(n) => Ok(*n),
        Expr::Var(v) => env.get(v).copied().ok_or_else(|| format!("undef {v}")),
        Expr::Eq(a, b) => Ok(if eval_expr(prog, a, env)? == eval_expr(prog, b, env)? {
            1
        } else {
            0
        }),
        Expr::If(c, t, f) => {
            if eval_expr(prog, c, env)? != 0 {
                eval_expr(prog, t, env)
            } else {
                eval_expr(prog, f, env)
            }
        }
        Expr::Call(n, args) => {
            let vs: Result<Vec<_>, _> = args.iter().map(|a| eval_expr(prog, a, env)).collect();
            eval_fun(prog, n, &vs?)
        }
    }
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Node {
    pub id: u32,
    pub kind: String,
    pub x: f32,
    pub y: f32,
    pub value: Bit,
    pub extra: HashMap<String, i64>,
}

#[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
pub struct Wire {
    pub from: u32,
    pub from_port: u8,
    pub to: u32,
    pub to_port: u8,
}

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Circuit {
    pub nodes: Vec<Node>,
    pub wires: Vec<Wire>,
}

pub struct Library {
    pub comps: HashMap<String, Program>,
}

impl Library {
    pub fn load_dir(dir: &Path) -> Result<Self, String> {
        let mut comps = HashMap::new();
        let rd = fs::read_dir(dir).map_err(|e| e.to_string())?;
        for e in rd.flatten() {
            let p = e.path();
            if p.extension().and_then(|s| s.to_str()) != Some("fun") {
                continue;
            }
            let name = p.file_stem().unwrap().to_string_lossy().to_string();
            let src = fs::read_to_string(&p).map_err(|e| e.to_string())?;
            comps.insert(name, parse_funo(&src)?);
        }
        Ok(Self { comps })
    }
}

impl Circuit {
    pub fn tick(&mut self, lib: &Library) {
        let inputs: HashMap<(u32, u8), Bit> = {
            let mut m = HashMap::new();
            for w in &self.wires {
                if let Some(src) = self.nodes.iter().find(|n| n.id == w.from) {
                    m.insert((w.to, w.to_port), src.value);
                }
            }
            m
        };
        for n in &mut self.nodes {
            let Some(prog) = lib.comps.get(&n.kind) else {
                continue;
            };
            let mut args = Vec::new();
            match n.kind.as_str() {
                "pin" => args.push(n.value as i64),
                "clk" => args.push(n.value as i64),
                "not" | "led" => args.push(*inputs.get(&(n.id, 0)).unwrap_or(&0) as i64),
                "dff" => {
                    args.push(*inputs.get(&(n.id, 0)).unwrap_or(&0) as i64);
                    args.push(*inputs.get(&(n.id, 1)).unwrap_or(&0) as i64);
                    args.push(*n.extra.get("prev_clk").unwrap_or(&0));
                    args.push(n.value as i64);
                }
                "mux" => {
                    args.push(*inputs.get(&(n.id, 0)).unwrap_or(&0) as i64);
                    args.push(*inputs.get(&(n.id, 1)).unwrap_or(&0) as i64);
                    args.push(*inputs.get(&(n.id, 2)).unwrap_or(&0) as i64);
                }
                _ => {
                    args.push(*inputs.get(&(n.id, 0)).unwrap_or(&0) as i64);
                    args.push(*inputs.get(&(n.id, 1)).unwrap_or(&0) as i64);
                }
            }
            if let Ok(v) = eval_fun(prog, "eval", &args) {
                if n.kind == "dff" {
                    n.extra
                        .insert("prev_clk".into(), *inputs.get(&(n.id, 1)).unwrap_or(&0) as i64);
                }
                if n.kind != "pin" {
                    n.value = if v != 0 { 1 } else { 0 };
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn and_gate() {
        let p = parse_funo(
            "fun eval(a, b) { if a == 1 then if b == 1 then return(1) else return(0) else return(0) }",
        )
        .unwrap();
        assert_eq!(eval_fun(&p, "eval", &[1, 1]).unwrap(), 1);
        assert_eq!(eval_fun(&p, "eval", &[1, 0]).unwrap(), 0);
    }
}
