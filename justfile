default: run_release

prefix := env("PREFIX", env("HOME") + "/.local")

run_release:
    cargo run --release -- images/wall2.jpg

release:
	cargo build --release

# Build and install binary, desktop entry and icon (default: ~/.local, override with PREFIX=/usr/local)
install: release
	install -Dm755 target/release/croppers "{{prefix}}/bin/croppers"
	install -Dm644 assets/croppers.desktop "{{prefix}}/share/applications/croppers.desktop"
	install -Dm644 assets/croppers.svg "{{prefix}}/share/icons/hicolor/scalable/apps/croppers.svg"
	-update-desktop-database "{{prefix}}/share/applications" 2>/dev/null
	-gtk-update-icon-cache -f -t "{{prefix}}/share/icons/hicolor" 2>/dev/null

uninstall:
	rm -f "{{prefix}}/bin/croppers"
	rm -f "{{prefix}}/share/applications/croppers.desktop"
	rm -f "{{prefix}}/share/icons/hicolor/scalable/apps/croppers.svg"
	-update-desktop-database "{{prefix}}/share/applications" 2>/dev/null
