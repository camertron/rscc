FROM archlinux:base-20240929.0.266368

RUN echo "[multilib]" >> /etc/pacman.conf && echo "Include = /etc/pacman.d/mirrorlist" >> /etc/pacman.conf
RUN pacman -Syu --noconfirm
RUN pacman -S wine wget --noconfirm
RUN pacman -S xorg-server-xvfb --noconfirm
RUN pacman -S samba --noconfirm
RUN rm -rf ~/.wine && mkdir -p ~/.wine
ENV WINEPREFIX=/root/.wine
ENV WINEARCH=win32
ENV WINEPATH=/root
RUN wineboot --init
RUN wget https://jrsoftware.org/download.php/is.exe?site=1 -O innosetup.exe
ENV DISPLAY=:0
RUN ((Xvfb :0 -screen 0 1024x768x16 &) && sleep 5 && wine innosetup.exe /SILENT /VERYSILENT /SUPPRESSMSGBOXES) || true
