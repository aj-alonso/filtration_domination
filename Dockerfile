# syntax=docker/dockerfile:1
FROM docker.io/rust:1.62-bullseye as rust-builder
WORKDIR /opt/filt
COPY Cargo.toml ./
COPY src ./src/
COPY experiments/experiment_runner/Cargo.toml experiments/experiment_runner/Cargo.lock ./experiments/experiment_runner/
COPY experiments/experiment_runner/src ./experiments/experiment_runner/src
WORKDIR /opt/filt/experiments/experiment_runner
RUN cargo build --profile release

FROM docker.io/buildpack-deps:bullseye as cpp-builder
RUN apt-get update && apt-get install -y cmake libgtest-dev libboost-test-dev git && rm -rf /var/lib/apt/lists/*

# Build mpfree
RUN git clone https://bitbucket.org/mkerber/mpfree.git
WORKDIR mpfree/build
RUN cmake -DCMAKE_BUILD_TYPE=Release .. && make

COPY experiments/single_parameter/CMakeLists.txt \
    experiments/single_parameter/single_parameter_glisse_pritam.cpp \
    experiments/single_parameter/Flag_complex_edge_collapser.h /single_parameter/
WORKDIR /single_parameter/build
RUN cmake -DCMAKE_BUILD_TYPE=Release .. && make

FROM docker.io/rust:1.62-bullseye
COPY --from=rust-builder /opt/filt/experiments/experiment_runner/target/release/experiment_runner /usr/local/bin/experiment_runner
COPY --from=cpp-builder /mpfree/build/mpfree /usr/local/bin/mpfree
COPY --from=cpp-builder /single_parameter/build/single_parameter /usr/local/bin/single_parameter
WORKDIR /opt/filt
