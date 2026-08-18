# D-триггер: по фронту clk защёлкивает d
fun eval(d, clk, prev_clk, q) {
    if clk == 1 then
        if prev_clk == 0 then return(d) else return(q)
    else
        return(q)
}
