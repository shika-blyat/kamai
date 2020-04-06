class ParseError(Exception):
    pass


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
                res += block_tokens
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
                res += block_tokens
            else:
                raise ParseError("Empty function not allowed")
            res.append("}")
        else:
            if tok_idx > level:
                res.append(tok)
            elif k + 1 < len(s) and s[k + 1][1] == level:
                print("a")
                res.append(";")
                res.append(tok)
            elif tok_idx == level:
                res.append(tok)
            else:
                return (k, res)
        k += 1
    return (k, res)


CODE = """a = b
              where b = c
                    c = d
          c = d"""


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


lex = lex_code(CODE)
print(lex)
print(convert(lex))
