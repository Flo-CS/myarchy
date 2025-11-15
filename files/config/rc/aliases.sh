alias ls='ls --color=auto'
alias grep='grep --color=auto'

alias lsa='ls --color=auto -la --group-directories-first'
alias shutdown='systemctl poweroff'

alias vi='nvim'
alias nv='nvim'

alias gs="git status"
alias ga="git add"
alias gc="git commit"
alias gp="git push"
alias gd="git diff"

#alias ls='eza -lh --group-directories-first --icons=auto'
#alias lsa='ls -a'
#alias lt='eza --tree --level=2 --long --icons --git'
#alias lta='lt -a'

open() {
	xdg-open "$@" >/dev/null 2>&1 &
}
