#!/bin/sh

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
