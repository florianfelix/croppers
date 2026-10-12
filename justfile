default: run_dev

home := env("HOME", env("USERPROFILE", ""))
prefix := env("PREFIX", home + "/.local")

run_release:
    cargo run --release

run_dev:
    cargo run --release -- images/wall2.jpg

release:
    cargo build --release

# Build and install on Linux (default: ~/.local, override with PREFIX=/usr/local)
[linux]
install: release
    install -Dm755 target/release/croppers "{{prefix}}/bin/croppers"
    install -Dm644 assets/croppers.desktop "{{prefix}}/share/applications/croppers.desktop"
    install -Dm644 assets/croppers.svg "{{prefix}}/share/icons/hicolor/scalable/apps/croppers.svg"
    -update-desktop-database "{{prefix}}/share/applications" 2>/dev/null
    -gtk-update-icon-cache -f -t "{{prefix}}/share/icons/hicolor" 2>/dev/null

[linux]
uninstall:
    rm -f "{{prefix}}/bin/croppers"
    rm -f "{{prefix}}/share/applications/croppers.desktop"
    rm -f "{{prefix}}/share/icons/hicolor/scalable/apps/croppers.svg"
    -update-desktop-database "{{prefix}}/share/applications" 2>/dev/null

# Build and install on Windows (places croppers.exe in %USERPROFILE%\.cargo\bin)
[windows]
install: release
    cargo install --path .

[windows]
uninstall:
    cargo uninstall croppers
