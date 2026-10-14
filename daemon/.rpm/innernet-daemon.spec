%define __spec_install_post %{nil}
%define __os_install_post %{_dbpath}/brp-compress
%define debug_package %{nil}

Name: innernet-daemon
Summary: A background management daemon for innernet interfaces.
Version: @@VERSION@@
Release: @@RELEASE@@%{?dist}
License: MIT
Source0: %{name}-%{version}.tar.gz
URL: https://github.com/tonarino/innernet

BuildRoot: %{_tmppath}/%{name}-%{version}-%{release}-root

Requires: glibc
Requires: systemd
Requires: libgcc

%description
%{summary}

%prep
%setup -q

%build

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}
cp -a * %{buildroot}

%clean
rm -rf %{buildroot}

%files
%defattr(-,root,root,-)
%{_bindir}/*
%attr(0644, root, root) "/usr/lib/systemd/system/innernet-daemon.service"
