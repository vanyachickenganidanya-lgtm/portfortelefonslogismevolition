import { parseFuno, evalFun } from "./funo.js";
import { FUNO_SRC } from "./funo-bundle.js";

const KINDS = ["and", "or", "not", "nand", "nor", "xor", "mux", "dff", "clk", "pin", "led"];
const PORTS = {
  and: 2, or: 2, nand: 2, nor: 2, xor: 2, not: 1, led: 1, pin: 0, clk: 0, mux: 3, dff: 2,
};

const lib = {};
const srcCache = {};

async function loadLib() {
  for (const k of KINDS) {
    let t = FUNO_SRC[k];
    if (!t) {
      for (const url of [`./funo/components/${k}.fun`, `../funo/components/${k}.fun`]) {
        try {
          const r = await fetch(url);
          if (r.ok) { t = await r.text(); break; }
        } catch (_) { /* file:// APK */ }
      }
    }
    if (!t) throw new Error("missing funo " + k);
    srcCache[k] = t;
    lib[k] = parseFuno(t);
  }
}

const state = {
  tool: "and",
  running: true,
  nextId: 10,
  nodes: [],
  wires: [],
  drag: null,
  history: [],
};

function node(id, kind, x, y, value = 0) {
  return { id, kind, x, y, value, extra: {} };
}

function demoXor() {
  state.nodes = [
    node(1, "pin", 50, 90, 1),
    node(2, "pin", 50, 220, 0),
    node(3, "xor", 180, 155, 0),
    node(4, "not", 280, 90, 0),
    node(5, "and", 280, 220, 0),
    node(6, "led", 370, 155, 0),
    node(7, "clk", 50, 330, 0),
  ];
  state.wires = [
    { from: 1, to: 3, tp: 0 },
    { from: 2, to: 3, tp: 1 },
    { from: 3, to: 6, tp: 0 },
    { from: 1, to: 4, tp: 0 },
    { from: 2, to: 5, tp: 0 },
    { from: 4, to: 5, tp: 1 },
  ];
}

function tick() {
  const inputs = new Map();
  for (const w of state.wires) {
    const src = state.nodes.find((n) => n.id === w.from);
    if (src) inputs.set(`${w.to}:${w.tp}`, src.value);
  }
  for (const n of state.nodes) {
    const prog = lib[n.kind];
    if (!prog) continue;
    let args = [];
    if (n.kind === "pin") args = [n.value];
    else if (n.kind === "clk") args = [n.value];
    else if (n.kind === "not" || n.kind === "led") args = [inputs.get(`${n.id}:0`) ?? 0];
    else if (n.kind === "dff") {
      args = [inputs.get(`${n.id}:0`) ?? 0, inputs.get(`${n.id}:1`) ?? 0, n.extra.prev_clk ?? 0, n.value];
    } else if (n.kind === "mux") {
      args = [inputs.get(`${n.id}:0`) ?? 0, inputs.get(`${n.id}:1`) ?? 0, inputs.get(`${n.id}:2`) ?? 0];
    } else {
      args = [inputs.get(`${n.id}:0`) ?? 0, inputs.get(`${n.id}:1`) ?? 0];
    }
    try {
      const v = evalFun(prog, "eval", args);
      if (n.kind === "dff") n.extra.prev_clk = inputs.get(`${n.id}:1`) ?? 0;
      if (n.kind !== "pin") n.value = v ? 1 : 0;
    } catch (e) {
      console.warn(n.kind, e);
    }
  }
  const snap = state.nodes.map((n) => `${n.kind[0]}${n.value}`).join(" ");
  state.history.push(snap);
  if (state.history.length > 10) state.history.shift();
  document.getElementById("chrono").textContent = state.history.join("\n");
}

const cv = document.getElementById("cv");
const ctx = cv.getContext("2d");

function resize() {
  const r = cv.getBoundingClientRect();
  const dpr = Math.min(devicePixelRatio || 1, 2);
  cv.width = r.width * dpr;
  cv.height = 420 * dpr;
  ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
}
addEventListener("resize", resize);

function pos(ev) {
  const r = cv.getBoundingClientRect();
  const t = ev.touches ? ev.touches[0] : ev;
  return { x: t.clientX - r.left, y: t.clientY - r.top };
}

function hit(p) {
  return state.nodes.find((n) => (n.x - p.x) ** 2 + (n.y - p.y) ** 2 < 26 ** 2);
}

cv.addEventListener("pointerdown", (e) => {
  cv.setPointerCapture(e.pointerId);
  const p = pos(e);
  const n = hit(p);
  if (n) {
    if (n.kind === "pin") {
      n.value ^= 1;
      document.getElementById("status").textContent = `PIN #${n.id} = ${n.value}`;
      return;
    }
    state.drag = { from: n, x: p.x, y: p.y };
    return;
  }
  const id = state.nextId++;
  state.nodes.push(node(id, state.tool, p.x, p.y, state.tool === "pin" ? 1 : 0));
  document.getElementById("status").textContent = `Поставлен ${state.tool.toUpperCase()} #${id}`;
});

cv.addEventListener("pointermove", (e) => {
  if (!state.drag) return;
  const p = pos(e);
  state.drag.x = p.x;
  state.drag.y = p.y;
});

cv.addEventListener("pointerup", (e) => {
  if (!state.drag) return;
  const n = hit(pos(e));
  if (n && n.id !== state.drag.from.id) {
    const tp = state.wires.filter((w) => w.to === n.id).length;
    state.wires.push({ from: state.drag.from.id, to: n.id, tp });
    document.getElementById("status").textContent = `Провод ${state.drag.from.kind} → ${n.kind}`;
  }
  state.drag = null;
});

function draw() {
  const w = cv.getBoundingClientRect().width;
  const h = 420;
  ctx.clearRect(0, 0, w, h);
  ctx.strokeStyle = "#1c2748";
  ctx.lineWidth = 1;
  for (let x = 0; x < w; x += 20) {
    ctx.beginPath(); ctx.moveTo(x, 0); ctx.lineTo(x, h); ctx.stroke();
  }
  for (let y = 0; y < h; y += 20) {
    ctx.beginPath(); ctx.moveTo(0, y); ctx.lineTo(w, y); ctx.stroke();
  }
  for (const wire of state.wires) {
    const a = state.nodes.find((n) => n.id === wire.from);
    const b = state.nodes.find((n) => n.id === wire.to);
    if (!a || !b) continue;
    ctx.strokeStyle = a.value ? "#3dff8a" : "#4a5878";
    ctx.lineWidth = a.value ? 3 : 2;
    ctx.beginPath();
    ctx.moveTo(a.x, a.y);
    ctx.bezierCurveTo((a.x + b.x) / 2, a.y, (a.x + b.x) / 2, b.y, b.x, b.y);
    ctx.stroke();
  }
  if (state.drag) {
    ctx.strokeStyle = "#6de8ff";
    ctx.setLineDash([6, 4]);
    ctx.beginPath();
    ctx.moveTo(state.drag.from.x, state.drag.from.y);
    ctx.lineTo(state.drag.x, state.drag.y);
    ctx.stroke();
    ctx.setLineDash([]);
  }
  for (const n of state.nodes) {
    ctx.beginPath();
    ctx.fillStyle = n.value ? "#1f8a55" : "#24325a";
    ctx.strokeStyle = n.value ? "#3dff8a" : "#6de8ff";
    ctx.lineWidth = 2;
    ctx.arc(n.x, n.y, 22, 0, Math.PI * 2);
    ctx.fill();
    ctx.stroke();
    ctx.fillStyle = "#eef4ff";
    ctx.font = "700 11px system-ui";
    ctx.textAlign = "center";
    ctx.textBaseline = "middle";
    ctx.fillText(n.kind.toUpperCase(), n.x, n.y);
  }
  requestAnimationFrame(draw);
}

const tools = document.getElementById("tools");
KINDS.forEach((k) => {
  const b = document.createElement("button");
  b.textContent = k.toUpperCase();
  b.onclick = () => {
    state.tool = k;
    [...tools.children].forEach((c) => c.classList.toggle("on", c === b));
    document.getElementById("funoSrc").textContent = srcCache[k] || "";
  };
  if (k === "and") b.classList.add("on");
  tools.appendChild(b);
});

document.getElementById("run").onclick = () => {
  state.running = !state.running;
  document.getElementById("run").textContent = state.running ? "⏸ пауза" : "▶ пуск";
};
document.getElementById("step").onclick = () => tick();
document.getElementById("clear").onclick = () => {
  state.nodes = [];
  state.wires = [];
  state.history = [];
};
document.getElementById("demo").onclick = demoXor;

await loadLib();
document.getElementById("funoSrc").textContent = srcCache.and;
demoXor();
resize();
draw();
setInterval(() => { if (state.running) tick(); }, 280);
