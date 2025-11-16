# Ripgrep with fzf for content search
# TODO: behavior is not very user friendly
rgf() {
	rg --line-number --no-heading --color=always --smart-case --hidden --glob '!.git' "${*:-}" |
		fzf --ansi \
			--scheme=default \
			--style full \
			--height 60% \
			--reverse \
			--delimiter : \
			--preview 'bat --color=always {1} --highlight-line {2}' \
			--preview-window "right:60%" \
			--bind 'enter:become(nvim {1} +{2})'
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
