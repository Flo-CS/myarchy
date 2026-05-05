#!/bin/sh
# Translate `idle-inhibitor watch` output into waybar JSON, one event per line.
# When the upstream watcher exits (daemon down or connection lost), emit an
# error state and sleep briefly so waybar's respawn isn't a tight loop.

idle-inhibitor watch | while IFS= read -r line; do
	case "$line" in
	"inhibited")
		printf '{"text": "☕", "tooltip": "Idle inhibited — click to toggle", "class": "active"}\n'
		;;
	"not inhibited")
		printf '{"text": "💤", "tooltip": "Idle not inhibited — click to toggle", "class": "inactive"}\n'
		;;
	esac
done

printf '{"text": "⚠", "tooltip": "idle-inhibitor daemon is not running", "class": "error"}\n'
sleep 5
