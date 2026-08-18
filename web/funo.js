/** Интерпретатор подмножества Funo (как в funo-studio). */

export function tokenize(src) {
  const out = [];
  let i = 0;
  while (i < src.length) {
    const c = src[i];
    if (c === "#") {
      while (i < src.length && src[i] !== "\n") i++;
      continue;
    }
    if (/\s/.test(c)) { i++; continue; }
    if (src.startsWith("==", i)) { out.push("=="); i += 2; continue; }
    if ("(){},=".includes(c)) { out.push(c); i++; continue; }
    const s = i;
    while (i < src.length && /[A-Za-z0-9_\-]/.test(src[i])) i++;
    if (s < i) out.push(src.slice(s, i));
    else i++;
  }
  return out;
}

export function parseFuno(src) {
  const toks = tokenize(src);
  let i = 0;
  const peek = () => toks[i];
  const eat = () => {
    if (i >= toks.length) throw new Error("eof");
    return toks[i++];
  };
  const expect = (w) => {
    const t = eat();
    if (t !== w) throw new Error(`expected ${w} got ${t}`);
  };
  function parseIf() {
    expect("if");
    const c = parseExpr();
    expect("then");
    const t = peek() === "if" ? parseIf() : peek() === "return" ? parseStmt() : parseExpr();
    let f = { type: "num", n: 0 };
    if (peek() === "else") {
      eat();
      f = peek() === "if" ? parseIf() : peek() === "return" ? parseStmt() : parseExpr();
    }
    return { type: "if", c, t, f };
  }
  function parseStmt() {
    if (peek() === "return") {
      eat(); expect("(");
      const e = parseExpr();
      expect(")");
      return e;
    }
    if (peek() === "if") return parseIf();
    return parseExpr();
  }
  function parseExpr() {
    const left = parseAtom();
    if (peek() === "==") {
      eat();
      return { type: "eq", a: left, b: parseAtom() };
    }
    return left;
  }
  function parseAtom() {
    const t = eat();
    if (/^-?\d+$/.test(t)) return { type: "num", n: +t };
    if (peek() === "(") {
      eat();
      const args = [];
      if (peek() !== ")") {
        for (;;) {
          args.push(parseExpr());
          if (peek() === ",") { eat(); continue; }
          break;
        }
      }
      expect(")");
      return { type: "call", name: t, args };
    }
    return { type: "var", name: t };
  }
  const funs = {};
  while (peek()) {
    expect("fun");
    const name = eat();
    expect("(");
    const params = [];
    if (peek() !== ")") {
      for (;;) {
        params.push(eat());
        if (peek() === ",") { eat(); continue; }
        break;
      }
    }
    expect(")");
    let body;
    if (peek() === "=") {
      eat();
      body = parseExpr();
    } else {
      expect("{");
      body = { type: "num", n: 0 };
      while (peek() !== "}") body = parseStmt();
      expect("}");
    }
    funs[name] = { params, body };
  }
  return { funs };
}

function ev(prog, e, env) {
  switch (e.type) {
    case "num": return e.n;
    case "var": return env[e.name] ?? 0;
    case "eq": return ev(prog, e.a, env) === ev(prog, e.b, env) ? 1 : 0;
    case "if": return ev(prog, e.c, env) ? ev(prog, e.t, env) : ev(prog, e.f, env);
    case "call": return evalFun(prog, e.name, e.args.map((a) => ev(prog, a, env)));
    default: return 0;
  }
}

export function evalFun(prog, name, args) {
  const f = prog.funs[name];
  if (!f) throw new Error("no fun " + name);
  const env = {};
  f.params.forEach((p, i) => { env[p] = args[i] ?? 0; });
  return ev(prog, f.body, env);
}
