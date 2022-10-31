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

# Build GUDHI
ADD https://github.com/GUDHI/gudhi-devel/releases/download/tags%2Fgudhi-release-3.6.0/gudhi.3.6.0.tar.gz /opt/
WORKDIR /opt
RUN tar -xzf gudhi.3.6.0.tar.gz && cd gudhi.3.6.0 && mkdir build && cd build && cmake .. && make install

# Build single-parameter utility.
COPY experiments/single_parameter/CMakeLists.txt \
    experiments/single_parameter/single_parameter_glisse_pritam.cpp /single_parameter/
WORKDIR /single_parameter/build
RUN cmake -DCMAKE_BUILD_TYPE=Release .. && make

FROM docker.io/rust:1.62-bullseye
COPY --from=rust-builder /opt/filt/experiments/experiment_runner/target/release/experiment_runner /usr/local/bin/experiment_runner
COPY --from=cpp-builder /mpfree/build/mpfree /usr/local/bin/mpfree
COPY --from=cpp-builder /single_parameter/build/single_parameter /usr/local/bin/single_parameter
WORKDIR /opt/filt
