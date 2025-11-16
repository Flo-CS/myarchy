# Ripgrep + fzf integration
rgf() {
	export TEMP=$(mktemp -u)
	trap 'rm -f "$TEMP"' EXIT

	INITIAL_QUERY="${*:-}"
	TRANSFORMER='
  rg_pat={q:1}      # The first word is passed to ripgrep
  fzf_pat={q:2..}   # The rest are passed to fzf

  if ! [[ -r "$TEMP" ]] || [[ $rg_pat != $(cat "$TEMP") ]]; then
    echo "$rg_pat" > "$TEMP"
    printf "reload:sleep 0.1; rg --column --line-number --no-heading --color=always --smart-case --hidden --glob "!.git" %q || true" "$rg_pat"
  fi
  echo "+search:$fzf_pat"
'
	fzf --ansi \
		--scheme=default \
		--style full \
		--height 60% \
		--reverse \
		--delimiter : \
		--disabled \
		--query "$INITIAL_QUERY" \
		--with-shell 'bash -c' \
		--bind "start,change:transform:$TRANSFORMER" \
		--color "hl:-1:underline,hl+:-1:underline:reverse" \
		--preview 'bat --color=always {1} --highlight-line {2}' \
		--preview-window 'up,60%,border-line,+{2}+3/3,~3' \
		--bind 'enter:become(vim {1} +{2})'

	history -s "rgf $*"
}

# Change directory using fzf
cdf() {
	local query=${1}
	local root=${2:-$(pwd)}

	local result=$(fd --type d -H . "$root" | fzf \
		--print-query \
		--query "$query" \
		--scheme=path \
		--style full \
		--height 60% \
		--reverse \
		--preview "tree -C {} 2>/dev/null || ls -la {}" \
		--preview-window=right:60%)

	local query=$(echo "$result" | sed -n '1p')
	local dir=$(echo "$result" | sed -n '2,$p')

	if [ -n "$dir" ]; then
		cd "$dir"
		history -s "cdf '$query' $root"
	fi
}

# View file with bat using fzf
batf() {
	local query=${1}
	local root=${2:-$(pwd)}

	local result=$(fd --type f -H . "$root" | fzf \
		--print-query \
		--query "$query" \
		--scheme=path \
		--style full \
		--height 60% \
		--reverse \
		--preview "bat --style=numbers --color=always {} 2>/dev/null || cat {}" \
		--preview-window=right:60%)

	local query=$(echo "$result" | sed -n '1p')
	local file=$(echo "$result" | sed -n '2,$p')

	if [ -n "$file" ]; then
		bat "$file"
		history -s "batf '$query' $root"
	fi
}

# Open file or directory in nvim using fzf
nvf() {
	local query=${1}
	local root=${2:-$(pwd)}

	local result=$(fd -H . "$root" | fzf \
		--print-query \
		--query "$query" \
		--scheme=path \
		--style full \
		--height 60% \
		--reverse \
		--preview "if [ -d {} ]; then tree -C {} 2>/dev/null || ls -la {}; else bat --style=numbers --color=always {} 2>/dev/null || cat {}; fi" \
		--preview-window=right:60%)

	local query=$(echo "$result" | sed -n '1p')
	local file=$(echo "$result" | sed -n '2,$p')

	if [ -d "$file" ]; then
		local before=$(pwd)
		cd "$file"
		nvim .
		cd "$before"
		history -s "nvf '$query' $root"
	elif [ -n "$file" ]; then
		nvim "$file"
		history -s "nvf '$query' $root"
	fi
}

# Search zoxide history with fzf
zf() {
	local dir=$(zoxide query -l "$1" | fzf \
		--scheme=path \
		--style full \
		--height 60% \
		--reverse \
		--preview "tree -C {} 2>/dev/null || ls -la {}" \
		--preview-window=right:60%)

	if [ -n "$dir" ]; then
		cd "$dir"
		history -s "cd $dir"
	fi
}

# TODO: move to bin
reboot-to-windows() {
	windows_title=$(sudo grep -i windows /boot/grub/grub.cfg | cut -d "'" -f 2)
	sudo grub-reboot "$windows_title" && sudo reboot
}
