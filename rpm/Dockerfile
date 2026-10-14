ARG DISTRO
ARG VER
ARG ARCH

FROM ${DISTRO}:${VER} as builder
LABEL stage=innernet-rpm

RUN yum -y update && \
	yum -y install gcc clang-devel sqlite-devel glibc-devel rpm-build libselinux-devel openssl-devel which && \
	(yum -y install webkit2gtk4.1-devel || yum -y install webkit2gtk3-devel || true) && \
	(yum -y install nodejs npm && npm install -g pnpm || true) && \
	curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal

WORKDIR /workdir
COPY . .
RUN rm -rf target

RUN source $HOME/.cargo/env && \
	cargo install cargo-rpm && \
	cargo build --release --verbose --features innernet/selinux && \
	# device::tests::test_add_peers will fail due to restricted docker env
	cargo test --release --verbose -- --skip test_add_peers && \
	cd server && cargo rpm build && \
	cd ../client && cargo rpm build && \
	(cd ../desktop && command -v pnpm >/dev/null 2>&1 && pnpm install && pnpm tauri build --bundles rpm || echo "Skipping desktop RPM")

# Copy RPMs to a temporary directory for better compatibility
RUN mkdir -p /target/rpm/${ARCH} && \
    cp -r /workdir/target/release/rpmbuild/RPMS/${ARCH}/* /target/rpm/${ARCH}/ && \
    (cp /workdir/desktop/src-tauri/target/release/bundle/rpm/*.rpm /target/rpm/${ARCH}/ 2>/dev/null || true)

FROM ${DISTRO}:${VER}
LABEL stage=innernet-rpm

RUN mkdir -p /target/rpm
COPY --from=builder /target/rpm/ /target/rpm/
