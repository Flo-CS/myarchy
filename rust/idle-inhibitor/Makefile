BINDIR  := $(HOME)/.local/bin
UNITDIR := $(HOME)/.config/systemd/user

.PHONY: build install uninstall clean

build:
	cargo build --release

install: build
	install -Dm755 target/release/idle-inhibitor  $(BINDIR)/idle-inhibitor
	install -Dm755 target/release/idle-inhibitord $(BINDIR)/idle-inhibitord
	install -Dm644 contrib/idle-inhibitor.user.service \
	    $(UNITDIR)/idle-inhibitor.service
	systemctl --user daemon-reload

uninstall:
	-systemctl --user disable --now idle-inhibitor.service
	rm -f $(BINDIR)/idle-inhibitor $(BINDIR)/idle-inhibitord
	rm -f $(UNITDIR)/idle-inhibitor.service
	systemctl --user daemon-reload

clean:
	cargo clean
