Name:       enpass-rust
Summary:    Enpass-rust
Version:    0.2.1
Release:    1
License:    LICENSE
Source0:    %{name}-%{version}.tar.bz2
Source1:    https://github.com/b100dian/enpass-rust/releases/download/%{version}/vendor-%{version}.tar.xz
Source2:    https://sqlite.org/src/raw/7dffa8cc89c7f2d73da4bd4ccea1bcbd2bd283e3bb4cea398df7c372a197291b?at=memvfs.c#/memvfs.c

Requires:   sqlcipher
BuildRequires:  sqlcipher-devel
BuildRequires:  rust >= 1.75
BuildRequires:  cargo >= 1.75
BuildRequires:  rust-std-static
BuildRequires:  sqlite-devel

%description
Command line enpass client written in rust.


%define BUILD_DIR "$PWD"/upstream/target

%prep
%setup -a1 -q -n %{name}-%{version}
%ifarch %arm32
%define SB2_TARGET armv7-unknown-linux-gnueabihf
%endif
%ifarch %arm64
%define SB2_TARGET aarch64-unknown-linux-gnu
%endif
%ifarch %ix86
%define SB2_TARGET i686-unknown-linux-gnu
%endif


# seems to need local stuff
%if 0%{?sailfishos_version}
tar -xJf %SOURCE1
%endif

# define the offline registry
%global cargo_home $PWD/.cargo
mkdir -p %{cargo_home}
cat > %{cargo_home}/config <<EOF
[source.crates-io]
replace-with = "vendored-sources"

[source.vendored-sources]
directory = "vendor"
EOF

# use our offline registry
export CARGO_HOME="%{cargo_home}"

%build

# Build libmemvfs.so
gcc -g -fPIC -shared %SOURCE2 -o libmemvfs.so

# Adopted from https://github.com/sailfishos/gecko-dev/blob/master/rpm/xulrunner-qt5.spec

export CARGO_HOME="%{BUILD_DIR}/cargo"
export CARGO_BUILD_TARGET=%SB2_TARGET

# When cross-compiling under SB2 rust needs to know what arch to emit
# when nothing is specified on the command line. That usually defaults
# to "whatever rust was built as" but in SB2 rust is accelerated and
# would produce x86 so this is how it knows differently. Not needed
# for native x86 builds
export SB2_RUST_TARGET_TRIPLE=%SB2_TARGET
export RUST_HOST_TARGET=%SB2_TARGET

export RUST_TARGET=%SB2_TARGET
export TARGET=%SB2_TARGET
export HOST=%SB2_TARGET
export SB2_TARGET=%SB2_TARGET

%ifarch %arm32 %arm64
export CROSS_COMPILE=%SB2_TARGET

# This avoids a malloc hang in sb2 gated calls to execvp/dup2/chdir
# during fork/exec. It has no effect outside sb2 so doesn't hurt
# native builds.
export SB2_RUST_EXECVP_SHIM="/usr/bin/env LD_PRELOAD=/usr/lib/libsb2/libsb2.so.1 /usr/bin/env"
export SB2_RUST_USE_REAL_EXECVP=Yes
export SB2_RUST_USE_REAL_FN=Yes
export SB2_RUST_NO_SPAWNVP=Yes
%endif

export CC=gcc
export CXX=g++
export AR="ar"
export NM="gcc-nm"
export RANLIB="gcc-ranlib"
export PKG_CONFIG="pkg-config"

#export RUSTFLAGS="-Clink-arg=-Wl,-z,relro,-z,now -Ccodegen-units=1 %{?rustflags}"
#export CARGO_INCREMENTAL=0
#
#export CRATE_CC_NO_DEFAULTS=1

export CARGOFLAGS=" --offline"
export CARGO_NET_OFFLINE=1"
export CARGO_BUILD_TARGET=armv7-unknown-linux-gnueabihf
export CARGO_CFG_TARGET_ARCH=arm"

cargo build --offline -j1 --release --target-dir=%{BUILD_DIR}

%install
mkdir -p %{buildroot}/%{_bindir}
install %{BUILD_DIR}/%{SB2_TARGET}/release/%{name} %{buildroot}/%{_bindir}/%{name}
install libmemvfs.so %{buildroot}/%{_libdir}/libmemvfs.so

%files
%defattr(-,root,root,-)
%defattr(0644,root,root,-)
%defattr(0755,root,root,-)
%{_bindir}/*
