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

set iskeyword=a-z,A-Z,_
syntax keyword langTodos TODO BUG NOTE

" Language Keywords 
syntax keyword langKeywords if if* else func let exit dump continue break for

" Comments 
syntax region langCommentLine start="//" end="$" contains=langTodos

" Strings
syntax region langString start=/\v"/ skip=/\v\\./ end=/\v"/ contains=langEscapes

" Chars
syntax region langChar start=/\v'/ skip=/\v\\./ end=/\v'/ contains=langEscapes

highlight default link langTodos Todo 
highlight default link langKeywords Keyword 
highlight default link langCommentLine Comment 
highlight default link langString String
highlight default link langChar Character

let b:current_syntax = "language"
