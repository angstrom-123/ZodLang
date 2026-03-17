" Usage
"
" Put this file in your config file (e.g. at: ~/.config/nvim/syntax/language.vim)
"
" VIM:
" Add this line to your .vimrc:
"       autocmd BufRead,BufNewFile *.lang set filetype=language
"
" NEOVIM:
" Add to a file at ~/.config/nvim/ftdetect/language.vim
"       au BufRead,BufNewFile *.zod set filetype=zodlang

if exists("b:current_syntax")
    finish
endif

set iskeyword=a-z,A-Z,_,48-57,94
syntax keyword zodTodos TODO BUG NOTE

" Types
syntax keyword zodTypes void i64 i64^ chr chr^ any^

" Language Keywords 
syntax keyword zodKeywords if if* else continue break for return while syscall include

" Comments 
syntax region zodCommentLine start="//" end="$" contains=zodTodos

" Strings
syntax region zodString start=/\v"/ skip=/\v\\./ end=/\v"/ contains=zodEscapes

" Chars
syntax region zodChar start=/\v'/ skip=/\v\\./ end=/\v'/ contains=zodEscapes

" Escapes 
syntax match zodEscapes display contained "\\[nr\"'0]"

highlight default link zodTodos Todo 
highlight default link zodTypes Type 
highlight default link zodKeywords Keyword 
highlight default link zodCommentLine Comment 
highlight default link zodString String
highlight default link zodChar Character
highlight default link zodEscapes SpecialChar

let b:current_syntax = "zodlang"
