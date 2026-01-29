use std::collections::HashMap;
use std::process::Command;
use crate::lexer::{Lexer, Token};
use crate::parser::{Parser, Stmt};

pub struct Generator {
    pub output: String,
    symbols: HashMap<String, String>,
    class_map: HashMap<String, Vec<String>>,
    obj_types: HashMap<String, String>,
    reg_count: usize,
    label_count: usize,
}

impl Generator {
    pub fn new() -> Self {
        Self {
            output: ".global _start\n.section .text\n\n_start:\n    mov x11, #10\n    mov x0, #0\n    mov x1, #4096\n    mov x2, #3\n    mov x3, #34\n    mov x4, #-1\n    mov x5, #0\n    mov x8, #222\n    svc #0\n    mov x20, x0\n".to_string(),
            symbols: HashMap::new(),
            class_map: HashMap::new(),
            obj_types: HashMap::new(),
            reg_count: 12,
            label_count: 0,
        }
    }

    fn get_path_info(&self, path: &Vec<String>) -> (String, usize) {
        let base_var = &path[0];
        let reg = self.symbols.get(base_var).cloned().unwrap_or("x0".to_string());
        let mut offset = 0;
        if path.len() > 1 {
            if let Some(c) = self.obj_types.get(base_var) {
                if let Some(fields) = self.class_map.get(c) {
                    offset = fields.iter().position(|f| f == &path[1]).unwrap_or(0) * 8;
                }
            }
        }
        (reg, offset)
    }

    pub fn generate(&mut self, ast: Vec<Stmt>) -> String {
        for s in ast { self.gen_stmt(s); }
        self.output.push_str("\n    mov x0, #0\n    mov x8, #93\n    svc #0\n");
        self.output.clone()
    }

    fn gen_stmt(&mut self, stmt: Stmt) {
        match stmt {
            Stmt::MergeBlock(content) => {
                let mut lexer = Lexer::new(content);
                let mut tokens = Vec::new();
                loop {
                    let t = lexer.next_token();
                    if t == Token::EOF { break; }
                    tokens.push(t);
                }
                let mut parser = Parser::new(tokens);
                let sub_ast = parser.parse_program();
                for s in sub_ast { self.gen_stmt(s); }
            }
            Stmt::PythonBlock(script) => {
                let out = Command::new("python3").arg("-c").arg(&script).output().expect("Python failed");
                let res = String::from_utf8_lossy(&out.stdout).to_string();
                self.output.push_str(&format!("\n    // Python Output: {}\n", res.trim()));
            }
            Stmt::IntelBlock(code) => {
                self.output.push_str("\n    .intel_syntax noprefix\n");
                self.output.push_str(&format!("    {}\n", code));
                self.output.push_str("    .att_syntax\n");
            }
            Stmt::AsmBlock(code) => { self.output.push_str(&format!("    {}\n", code)); }
            Stmt::ProbIf { chance, body } => {
                let id = self.label_count; self.label_count += 1;
                let math_reg = self.symbols.get("math").cloned().unwrap_or("x12".into());
                self.output.push_str(&format!("\n    // Chaos Roll {}%\n    ldr x1, [{}, #8]\n", chance, math_reg));
                
                self.output.push_str(&format!("    cmp x1, #0\n    b.ne .Lskp{}\n    mrs x1, cntvct_el0\n.Lskp{}:\n", id, id));
                
                self.output.push_str("    ldr x2, =0x9E3779B97F4A7C15\n    mul x1, x1, x2\n    eor x1, x1, x1, lsr #33\n");
                self.output.push_str(&format!("    str x1, [{}, #8]\n", math_reg));
                self.output.push_str("    and x1, x1, #0x7FFFFFFF\n    mov x2, #100\n    udiv x3, x1, x2\n    msub x1, x3, x2, x1\n");
                self.output.push_str(&format!("    cmp x1, #{}\n    b.hs .Lif{}\n", chance as i64, id));
                for s in body { self.gen_stmt(s); }
                self.output.push_str(&format!(".Lif{}:\n", id));
            }
            Stmt::IfStmt { path, op, rhs_val, body } => {
                let id = self.label_count; self.label_count += 1;
                let (reg, offset) = self.get_path_info(&path);
                if path.len() > 1 {
                    self.output.push_str(&format!("    ldr x1, [{}, #{}]\n", reg, offset));
                } else {
                    self.output.push_str(&format!("    mov x1, {}\n", reg));
                }
                let cond = match op {
                    Token::Equal => "ne",
                    Token::Greater => "le",
                    Token::Less => "ge",
                    _ => "eq",
                };
                self.output.push_str(&format!("    cmp x1, #{}\n    b.{} .Lif{}\n", rhs_val as i64, cond, id));
                for s in body { self.gen_stmt(s); }
                self.output.push_str(&format!(".Lif{}:\n", id));
            }
            Stmt::WhileStmt { path, op, rhs_val, body } => {
                let id = self.label_count; self.label_count += 1;
                self.output.push_str(&format!(".Lw_start{}:\n", id));
                let (reg, offset) = self.get_path_info(&path);
                if path.len() > 1 {
                    self.output.push_str(&format!("    ldr x1, [{}, #{}]\n", reg, offset));
                } else {
                    self.output.push_str(&format!("    mov x1, {}\n", reg));
                }
                let cond = match op {
                    Token::Equal => "ne",
                    Token::Greater => "le",
                    Token::Less => "ge",
                    _ => "eq",
                };
                self.output.push_str(&format!("    cmp x1, #{}\n    b.{} .Lw_end{}\n", rhs_val as i64, cond, id));
                for s in body { self.gen_stmt(s); }
                self.output.push_str(&format!("    b .Lw_start{}\n.Lw_end{}:\n", id, id));
            }
            Stmt::LocalAssign { name, value } => {
                let reg = self.symbols.entry(name.clone()).or_insert_with(|| {
                    let r = format!("x{}", self.reg_count); self.reg_count += 1; r
                }).clone();
                self.output.push_str(&format!("    mov {}, #{}\n", reg, value as i64));
            }
            Stmt::FieldAssign { path, value } => {
                let (reg, offset) = self.get_path_info(&path);
                if path.len() > 1 {
                    self.output.push_str(&format!("    mov x1, #{}\n    str x1, [{}, #{}]\n", value as i64, reg, offset));
                } else {
                    self.output.push_str(&format!("    mov {}, #{}\n", reg, value as i64));
                }
            }
            Stmt::FieldMath { path, op, rhs_val } => {
                let (reg, offset) = self.get_path_info(&path);
                let instr = match op {
                    Token::Plus => "add",
                    Token::Minus => "sub",
                    _ => "add",
                };
                if path.len() > 1 {
                    self.output.push_str(&format!("    ldr x1, [{}, #{}]\n    {} x1, x1, #{}\n    str x1, [{}, #{}]\n", reg, offset, instr, rhs_val as i64, reg, offset));
                } else {
                    self.output.push_str(&format!("    {} {}, {}, #{}\n", instr, reg, reg, rhs_val as i64));
                }
            }
            Stmt::PrintVar(name) => {
                if let Some(reg) = self.symbols.get(&name).cloned() {
                    let id = self.output.len();
                    self.output.push_str(&format!("
    stp x0, x1, [sp, #-16]!
    mov x0, {}
    sub sp, sp, #32
    mov x1, sp
    add x1, x1, #31
    mov w2, #10
    strb w2, [x1]
.Lp{}:
    sub x1, x1, #1
    udiv x2, x0, x11
    msub x3, x2, x11, x0
    add x3, x3, #48
    strb w3, [x1]
    mov x0, x2
    cbnz x0, .Lp{}
    mov x0, #1
    mov x2, sp
    add x2, x2, #32
    sub x2, x2, x1
    mov x8, #64
    svc #0
    add sp, sp, #32
    ldp x0, x1, [sp], #16\n", reg, id, id));
                }
            }
            Stmt::PrintString(s) => {
                let id = self.label_count; self.label_count += 1;
                self.output.push_str(&format!("\n.section .data\n.Lstr{}: .ascii \"{}\\n\"\n.section .text\n", id, s));
                self.output.push_str(&format!("    mov x0, #1\n    adr x1, .Lstr{}\n    mov x2, #{}\n    mov x8, #64\n    svc #0\n", id, s.len() + 1));
            }
            Stmt::ClassDef { name, fields } => { self.class_map.insert(name, fields); }
            Stmt::HeapAlloc { var_name, class_name } => {
                let reg = format!("x{}", self.reg_count); self.reg_count += 1;
                self.symbols.insert(var_name.clone(), reg.clone());
                self.obj_types.insert(var_name, class_name.clone());
                if let Some(f) = self.class_map.get(&class_name) {
                    self.output.push_str(&format!("    mov {}, x20\n    add x20, x20, #{}\n", reg, f.len() * 8));
                }
            }
            _ => {}
        }
    }
}