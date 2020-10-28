# Maintainer: Lyr <lyr-7d1h@pm.me>

pkgname="dimport"
pkgver=1.0
pkgrel=1
pkgdesc="Dotfiles Importer - Import and keep your dotfiles in sync"
arch=('x86_64')
url="https://github.com/Lyr-7D1h/dotfiles_importer_revamped"
license=('MIT')
makedepends=('rust' 'cargo' 'dbus')

build() {
    cd "$srcdir/dimport"
    cargo build --release --locked 
    cd "$srcdir/dimportd"
    cargo build --release --locked
}

package() {
    install -D -m755 "$srcdir/dimport/target/release/dimport" "$pkgdir/usr/bin/dimport"
    install -D -m755 "$srcdir/dimportd/target/release/dimportd" "$pkgdir/usr/sbin/dimportd"
    install -D -m700 "$srcdir/../dimportd.socket" "$pkgdir/usr/lib/systemd/system"
    install -D -m700 "$srcdir/../dimportd.service" "$pkgdir/usr/lib/systemd/system"
    install -D -m644 "$srcdir/../LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
    mkdir -p "$pkgdir/etc/dimport"
    mkdir -p "$pkgdir/var/lib/dimport"
}