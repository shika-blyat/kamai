'+' means one or more

'|' is a logical or

'x'.'y' is a range

'-' specify an exception

'[' foo ']' is an optional part of the pattern

```
char = '0020' . '10FFFF' - '"' - '\'

ws = "" | '0020' ws | '000A' ws | '000D' ws | '0009' ws
ws1 = '0020' ws | '000A' ws | '000D' ws | '0009' ws

bool = "true" | "false"

digit = '1'.'9'

int = digit+

character = char |  '\' ('n' | 'r' | 't' | '\' | 'u')

characters = character+

string = "\"" characters "\""

atom = int | string | bool

operator = ('.' | '/' | '<' | '>' | '-' | ';' | ',' | ':' | '$' | '*' | '+' | '#' | '~'
        | '^' | '@' | '!' | '%')+

letter = 'a'.'Z'

idents = (digit | letter | '_' | '\'')+

identifier = letter idents

let = "let " ws identifier ws "=" ws expr

call = identifier ( "(" (expr ws "," ws )+ ")" | (expr ws1)+)

function = "fun" ws identifier "(" ws (identifier [ws ":" ws identifier] ws ",")+ ws ")" ws block

lambda = "fun" ws  "(" ws ( identifier [ ws ":" ws identifier ] ws "," )+ ws ")" ws block

condition = "if" ws1 expr ws block [ ws "else if" ws1 expr ws block ]+ [ ws "else" ws block ]

operation = expr (ws operator ws expr)+

expr = let | call | atom | lambda | operation | condition | "()"

instructions = (ws expr ws ";")+

block = "{" instructions ws "}"

program = ws (function ws)+ ws
```