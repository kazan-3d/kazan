.PHONY: all always_build clean

all: shader.o

always_build:

INITIAL_OPT := -inline -mem2reg -sccp -dce -simplifycfg

shader.bc: shader.cpp
	clang++ -std=c++14 -march=corei7 -Wall -o shader.ll shader.cpp -emit-llvm -S && opt $(INITIAL_OPT) shader.ll > shader.bc && llvm-dis shader.bc

shader.o: shader.bc always_build
	time -p bash -c 'for i in {1..10}; do echo $$i; opt -O3 < shader.bc | llc -filetype=obj -O3 -o shader.o; done'

clean:
	rm -f shader.bc shader.o shader.ll

