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
"       au BufRead,BufNewFile *.lang set filetype=language

if exists("b:current_syntax")
    finish
endif

set iskeyword=a-z,A-Z,_,48-57
syntax keyword langTodos TODO BUG NOTE

" Types
syntax keyword langTypes i64

" Language Keywords 
syntax keyword langKeywords if if* else exit dump continue break for return 

" Comments 
syntax region langCommentLine start="//" end="$" contains=langTodos

" Strings
syntax region langString start=/\v"/ skip=/\v\\./ end=/\v"/ contains=langEscapes

" Chars
syntax region langChar start=/\v'/ skip=/\v\\./ end=/\v'/ contains=langEscapes

highlight default link langTodos Todo 
highlight default link langTypes Type 
highlight default link langKeywords Keyword 
highlight default link langCommentLine Comment 
highlight default link langString String
highlight default link langChar Character

let b:current_syntax = "language"
