BINDIR  := $(HOME)/.local/bin
UNITDIR := $(HOME)/.config/systemd/user

.PHONY: build install uninstall clean

build:
	cargo build --release

install: build
	install -Dm755 target/release/myarchy-idle-inhibitord $(BINDIR)/myarchy-idle-inhibitord
	install -Dm644 contrib/myarchy-idle-inhibitor.user.service \
	    $(UNITDIR)/myarchy-idle-inhibitor.service
	systemctl --user daemon-reload

uninstall:
	-systemctl --user disable --now myarchy-idle-inhibitor.service
	rm -f $(BINDIR)/myarchy-idle-inhibitord
	rm -f $(UNITDIR)/myarchy-idle-inhibitor.service
	systemctl --user daemon-reload

clean:
	cargo clean
