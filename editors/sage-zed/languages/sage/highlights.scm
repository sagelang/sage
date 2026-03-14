; Comments
(line_comment) @comment
(block_comment) @comment

; Keywords - agent related
"agent" @keyword
"belief" @keyword
"on" @keyword
"use" @keyword

; Keywords - function related
"fn" @keyword
"return" @keyword

; Keywords - type definitions
"record" @keyword
"enum" @keyword
"tool" @keyword

; Keywords - control flow
"if" @keyword
"else" @keyword
"for" @keyword
"in" @keyword
"while" @keyword
"match" @keyword

; Keywords - other
"let" @keyword
"mut" @keyword
"run" @keyword
"emit" @keyword
"try" @keyword
"await" @keyword
"infer" @keyword
"pub" @keyword
"and" @keyword
"or" @keyword

; Handler events
(handler_event) @keyword

; Break and continue
(break_statement) @keyword
(continue_statement) @keyword

; Type annotations
(parameter type: (identifier) @type)
(record_field type: (identifier) @type)
(belief_declaration type: (identifier) @type)
(let_statement type: (identifier) @type)

; Generic types
(generic_type (identifier) @type)

; Declaration names
(agent_declaration name: (identifier) @type)
(record_declaration name: (identifier) @type)
(enum_declaration name: (identifier) @type)
(tool_declaration name: (identifier) @type)
(function_declaration name: (identifier) @function)
(tool_function name: (identifier) @function)

; Enum variants
(enum_variant (identifier) @constant)

; Beliefs and fields
(belief_declaration name: (identifier) @property)
(record_field name: (identifier) @property)

; Parameters
(parameter name: (identifier) @variable.parameter)

; Use clause (tool names)
(use_clause (identifier) @type)

; Variables
(identifier) @variable

; Operators
"+" @operator
"-" @operator
"*" @operator
"/" @operator
"%" @operator
"=" @operator
"==" @operator
"!=" @operator
"<" @operator
">" @operator
"<=" @operator
">=" @operator
"!" @operator
"=>" @operator
"->" @operator

; Punctuation
"(" @punctuation.bracket
")" @punctuation.bracket
"[" @punctuation.bracket
"]" @punctuation.bracket
"{" @punctuation.bracket
"}" @punctuation.bracket
"|" @punctuation.bracket
":" @punctuation.delimiter
"," @punctuation.delimiter
"." @punctuation.delimiter

; Literals
(integer) @number
(float) @number
(boolean) @constant.builtin

; Strings
(string) @string
(interpolated_string) @string
(escape_sequence) @escape
