# AND — выход 1 только если оба входа 1
fun eval(a, b) {
    if a == 1 then
        if b == 1 then return(1) else return(0)
    else
        return(0)
}
