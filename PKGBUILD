# Maintainer: Lyr <lyr-7d1h@pm.me>

pkgname="dimport"
pkgver=1.0
pkgrel=1
pkgdesc="Dotfiles Importer - Import and keep your dotfiles in sync"
arch=('x86_b64')
url="https://github.com/Lyr-7D1h/dotfiles_importer_revamped"
license=('MIT')
makedepends=('rust' 'cargo')

build() {
    cd src/dimport
    cargo build --release --locked 
    cd ../dimportd
    cargo build --release --locked
    
}

package() {
    install -D -m755 "src/dimport/target/release/dimport" "$pkgdir/usr/bin/dimport"
    install -D -m755 "src/dimportd/target/release/dimportd" "$pkgdir/usr/sbin/dimportd"
    install -D -m700 "dimportd.socket" "$pkgdir/usr/lib/systemd/system"
    install -D -m700 "dimportd.service" "$pkgdir/usr/lib/systemd/system"
    install -D -m644 "LICENSE" "$pkgdir/usr/share/licenses/$pkgname/LICENSE"
    mkdir /etc/dimport
    mkdir /var/lib/dimport
    # mkdir for config state repository and backup
}