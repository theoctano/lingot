" Vim syntax file
" Language: Lingot
" Maintainer: theoctano
" Latest Revision: 2026-03-16

if exists("b:current_syntax")
  finish
endif

" Keywords
syn keyword lingotKeyword let dyn pub
syn keyword lingotControl if else while repeat for in try catch return fail load from
syn keyword lingotObject Object

" Operator keywords
syn keyword lingotOperatorKw and or not is greater lesser than equal

" Boolean constants
syn keyword lingotBoolean true false

" Type names
syn keyword lingotType Text Number Bool List Void

" Built-in functions
syn keyword lingotBuiltin display shell read write move rename delete list prefix suffix
  \ nextgroup=lingotParenRegion

" Numbers
syn match lingotFloat "\<\d\+\.\d\+\>"
syn match lingotNumber "\<\d\+\>"

" Strings with interpolation
syn region lingotString start=+"+ end=+"+ skip=+\\\\\|\\"+ contains=lingotEscape,lingotInterpolation
syn match lingotEscape "\\[nrt\\"{}]" contained
syn region lingotInterpolation start="{" end="}" contained contains=TOP

" Comments
syn match lingotComment "//.*$"

" Operators
syn match lingotOperator "[+\-*/%]"
syn match lingotOperator "&&\|||"
syn match lingotOperator "[!=<>]="
syn match lingotOperator "[<>!]"
syn match lingotOperator "\.\."
syn match lingotOperator "="

" Function declarations: let name(...)
syn match lingotFuncDecl "\<let\>\s\+\%(dyn\s\+\)\?\%(pub\s\+\)\?\zs[a-zA-Z_][a-zA-Z0-9_]*\ze\s*(" contained containedin=TOP

" Function calls
syn match lingotFuncCall "\<[a-zA-Z_][a-zA-Z0-9_]*\ze\s*("

" Highlighting
hi def link lingotKeyword    Keyword
hi def link lingotControl    Conditional
hi def link lingotObject     Type
hi def link lingotOperatorKw Keyword
hi def link lingotBoolean    Boolean
hi def link lingotType       Type
hi def link lingotBuiltin    Function
hi def link lingotFloat      Float
hi def link lingotNumber     Number
hi def link lingotString     String
hi def link lingotEscape     SpecialChar
hi def link lingotInterpolation Special
hi def link lingotComment    Comment
hi def link lingotOperator   Operator
hi def link lingotFuncDecl   Function
hi def link lingotFuncCall   Function

let b:current_syntax = "lingot"
