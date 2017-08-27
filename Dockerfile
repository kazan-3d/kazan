FROM ubuntu:xenial
RUN apt-get update -q && apt-get dist-upgrade -qy && apt-get install -qy clang-4.0 build-essential cmake ninja-build llvm-4.0-dev libsdl2-dev curl imagemagick && apt-get clean -y
WORKDIR /build
COPY . /build
RUN ./docker-build-scripts/build.sh
RUN ./build/src/demo/demo && convert output.bmp output.png && curl --upload-file ./output.png https://transfer.sh/output.png || echo running failed
CMD ["/bin/bash"]
