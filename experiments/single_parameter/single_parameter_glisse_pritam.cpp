#include <vector>
#include <iostream>
#include <chrono>

#include "Flag_complex_edge_collapser.h"

using uu = unsigned long int;

void write_edges(const std::vector<std::tuple<uu, uu, double> > &edges) {
  std::ofstream out("edges_out.txt");
  out.precision(std::numeric_limits<double>::max_digits10);
  out << edges.size() << std::endl;
  for (auto &[u, v, d] : edges) {
    out << u << " " << v << " " << d << std::endl;
  }
}

int main(int argc, char** argv) {
	if (argc < 2) {
		std::cout << "Please give me an edge list" << std::endl;
		std::exit(1);
	}

	std::ifstream in(argv[1], std::ios_base::in);

	int n_edges;
	std::vector<std::tuple<uu, uu, double>> edges;

	in >> n_edges;
	for (int i = 0; i < n_edges; i++) {
		std::tuple<uu, uu, double> e;
		in >> std::get<0>(e) >> std::get<1>(e) >> std::get<2>(e);
		edges.push_back(e);
	}

	std::cerr << "Original edges: " << edges.size() << std::endl;

	std::chrono::steady_clock::time_point begin = std::chrono::steady_clock::now();
	auto collapsed_edges = Gudhi::collapse::flag_complex_collapse_edges(edges);
	std::chrono::steady_clock::time_point end = std::chrono::steady_clock::now();
	auto elapsed_collapse = (std::chrono::duration_cast<std::chrono::microseconds>(end - begin).count()) / 1000000.0;

        write_edges(collapsed_edges);

	std::cerr << "Collapsed edges: " << collapsed_edges.size() << std::endl;
	std::cerr << "Time taken to collapse (sec) = " << elapsed_collapse << std::endl;

	std::cout << collapsed_edges.size() << std::endl;
	std::cout << elapsed_collapse << std::endl;

	return 0;
}
