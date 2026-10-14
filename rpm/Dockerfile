ARG DISTRO
ARG VER
ARG ARCH

FROM ${DISTRO}:${VER} as builder
ARG ARCH
LABEL stage=innernet-rpm

RUN yum -y update && \
	yum -y install gcc clang-devel sqlite-devel glibc-devel rpm-build libselinux-devel openssl-devel which pkgconfig && \
	(yum -y install gtk3-devel webkit2gtk4.1-devel libappindicator-gtk3-devel librsvg2-devel || yum -y install gtk3-devel webkit2gtk4.0-devel || true) && \
	(yum -y install nodejs npm && npm install -g pnpm || true) && \
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal

WORKDIR /workdir
COPY . .
RUN rm -rf target desktop/src-tauri/target desktop/node_modules desktop/dist

RUN source $HOME/.cargo/env && \
	cargo install cargo-rpm && \
	cargo build --release --verbose --features innernet/selinux && \
	# device::tests::test_add_peers will fail due to restricted docker env
	cargo test --release --verbose -- --skip test_add_peers && \
	cd server && cargo rpm build && \
	cd ../client && cargo rpm build && \
	cd ../desktop && \
	if command -v pnpm >/dev/null 2>&1 && pkg-config --exists gtk+-3.0; then \
		echo "Building innernet-desktop RPM..." && \
		pnpm install && \
		pnpm tauri build --bundles rpm; \
	else \
		echo "Desktop dependencies not available on this base image, skipping desktop RPM"; \
	fi

# Copy RPMs to a temporary directory for better compatibility
RUN mkdir -p /target/rpm/${ARCH} && \
    cp -r /workdir/target/release/rpmbuild/RPMS/${ARCH}/* /target/rpm/${ARCH}/ && \
    if [ -d /workdir/desktop/src-tauri/target/release/bundle/rpm ]; then \
        cp -v /workdir/desktop/src-tauri/target/release/bundle/rpm/*.rpm /target/rpm/${ARCH}/; \
    fi

FROM ${DISTRO}:${VER}
ARG ARCH
LABEL stage=innernet-rpm

RUN mkdir -p /target/rpm
COPY --from=builder /target/rpm/ /target/rpm/
