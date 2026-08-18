export const FUNO_SRC = {
  "and": "# AND — выход 1 только если оба входа 1\nfun eval(a, b) {\n    if a == 1 then\n        if b == 1 then return(1) else return(0)\n    else\n        return(0)\n}\n",
  "clk": "# Тактовый генератор: инвертирует себя каждый тик симуляции\nfun eval(q) {\n    if q == 1 then return(0) else return(1)\n}\n",
  "dff": "# D-триггер: по фронту clk защёлкивает d\nfun eval(d, clk, prev_clk, q) {\n    if clk == 1 then\n        if prev_clk == 0 then return(d) else return(q)\n    else\n        return(q)\n}\n",
  "led": "# Индикатор: просто повторяет вход\nfun eval(a) {\n    return(a)\n}\n",
  "mux": "# 2:1 мультиплексор. s выбирает a (0) или b (1)\nfun eval(a, b, s) {\n    if s == 1 then return(b) else return(a)\n}\n",
  "nand": "# NAND\nfun eval(a, b) {\n    if a == 1 then\n        if b == 1 then return(0) else return(1)\n    else\n        return(1)\n}\n",
  "nor": "# NOR\nfun eval(a, b) {\n    if a == 1 then return(0)\n    if b == 1 then return(0)\n    return(1)\n}\n",
  "not": "# NOT — инверсия\nfun eval(a) {\n    if a == 1 then return(0) else return(1)\n}\n",
  "or": "# OR — выход 1 если хотя бы один вход 1\nfun eval(a, b) {\n    if a == 1 then return(1)\n    if b == 1 then return(1)\n    return(0)\n}\n",
  "pin": "# Входной пин: значение задаёт пользователь\nfun eval(v) {\n    return(v)\n}\n",
  "xor": "# XOR\nfun eval(a, b) {\n    if a == b then return(0) else return(1)\n}\n"
};
