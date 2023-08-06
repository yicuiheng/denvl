grammar Calc;

prog: BLANK? expr+;

expr: let_expr | arith_expr;

// let x = 42; hoge
let_expr: LET IDENT EQUAL expr SEMICOLON expr;

arith_expr:
	arith_expr (AST | SLASH) arith_expr
	| arith_expr (PLUS | MINUS) arith_expr
	| INT
	| IDENT
	| LEFT_PAREN expr RIGHT_PAREN;

WHITE_SPACE: [ \t\r\n]+;
LINE_COMMENT: '//' ~[\r\n]* '\r'? '\n';
BLOCK_COMMENT: '/*' ('/' ~('*') | ~('/'))* '*/';
BLANK: (WHITE_SPACE | LINE_COMMENT | BLOCK_COMMENT)+;

INT: [0-9]+ BLANK?;
LET: 'let' BLANK?;
IDENT: [a-zA-Z][a-zA-Z0-9]* BLANK?;
EQUAL: '=' BLANK?;
AST: '*' BLANK?;
SLASH: '/' BLANK?;
PLUS: '+' BLANK?;
MINUS: '-' BLANK?;
SEMICOLON: ';' BLANK?;
LEFT_PAREN: '(' BLANK?;
RIGHT_PAREN: ')' BLANK?;
