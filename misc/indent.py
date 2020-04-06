class ParseError(Exception):
    pass


# Add special case for where block, allowing
# ```haskell
# add x y = z
#   where z = x + y
# ```
# to be parsed like correctly :)


def convert(s, level=1):
    k = 0
    res = []
    while k < len(s):
        tok, tok_idx = s[k]
        if tok == "where":
            res.append("where")
            res.append("{")
            if k + 1 < len(s):
                _, next_idx = s[k + 1]
                nk, block_tokens = convert(s[k + 1 :], next_idx)
                k += nk
                res += block_tokens[1:]
            else:
                raise ParseError("Empty where statement not allowed")
            res.append("}")
        elif tok == "=":
            res.append("=")
            res.append("{")
            if k + 1 < len(s):
                _, next_idx = s[k + 1]
                nk, block_tokens = convert(s[k + 1 :], next_idx)
                k += nk
                res += block_tokens[1:]
            else:
                raise ParseError("Empty function not allowed")
            res.append("}")
        else:
            if tok_idx > level:
                res.append(tok)
            elif tok_idx == level:
                res.append(";")
                res.append(tok)
            else:
                return (k, res)
        k += 1
    return (k, res)


CODE = """
a = b
  where b = z
        c = k
         where k = 2

"""


def lex_code(s):
    res = []
    current = ""
    cntr = 1
    for k, i in enumerate(s):
        if i.isspace():
            if current != "":
                res.append((current, cntr))
                current = ""
            if i == "\n":
                cntr = 0
        else:
            current += i
        if k == len(s) - 1:
            res.append((current, cntr + 1))
        cntr += 1
    return res


def pretty_print(l, k=0, level=0):
    """
    A (not really) pretty printer
    """
    s = ""
    while k < len(l):
        if l[k] == "{":
            s += " " * level + "{\n"
            (k, ns) = pretty_print(l, k + 1, level + 1)
            s += ns + "\n"
            s += " " * (level) + "}"
        elif l[k] == "}":
            return (k, s)
        else:
            s += " " * level + l[k] + " "
        k += 1
    return (k, s)


lex = lex_code(CODE)
print(lex)
print(pretty_print(convert(lex)[1])[1])
