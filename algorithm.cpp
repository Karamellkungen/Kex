#include <iostream>
#include <vector>
#include <fstream>
#include <random>
#include <string>
#include <math.h>
#include <algorithm>
#include <bits/stdc++.h>

template<class Node> using Graph = std::vector<Node>;
using Coloring = int[T];

#define LAMBDA 2.0

/*
*   Sample from a levy distribution with inverse transform sampling
*/
double levy(double c) {
    std::default_random_engine rand_gen;
    std::normal_distribution<double> normal(0.0, 1.0);
    return c / pow(normal(rand_gen), 2);
}

/*
*   Read and parse a graph from file
*/
Graph<Node> read_graph(char* fileName) {
    std::string line;
    std::ifstream file(fileName);
    Graph<Node> graph;
    // remove top part of file
    for (int i = 0; i < 7; i++) {
        std::getline(file, line);
    }
    int nodes;
    std::string temp;
    std::istringstream ss(line);
    ss >> temp;
    ss >> temp;
    ss >> nodes;
    for (int i = 0; i <= nodes; i++) {
        graph.push_back(Node(i, -1));
    }
    
    // read all edges
    while(std::getline(file, line)) {
        int from;
        int to;
        std::istringstream ss(line);
        
        ss >> temp;
        ss >> from;
        ss >> to;
        graph[from].neighbors.push_back(to);
        graph[to].neighbors.push_back(from);
    } // vet inte om det är rätt eller inte men vi får se

    return graph;
}

class Graph {
    private:
        std::vector<std::vector<int>> graph;
    public:
        Graph(std::vector<std::vector<int>> g) {
            graph = g;
        }
}

class Node {
public:
// private:
    int identity; // needed?
    int color;
    std::vector<int> neighbors;

    Node(int ident, int col) : color(col), identity(ident) {
    }

    Node(const Node &original) {
        identity = original.identity;
        color = original.color;
        neighbors = original.neighbors;
    }
};

/*
*   Calculate the max degree of a given graph
*/
int max_degree(Graph<Node> graph) {
    int max = 0;
    for (Node n : graph) {
        max = std::max(max, n.neighbours.length);
    }
    return max;
}

/*  
*   TODO
*   Initialize a population with random colorings
*/
std::vector<Coloring> populate(Graph<Node> graph, int n, int k) {
    std::vector<Coloring> population;
    int nodes = graph.size();
    for (int i = 0; i < n; i++) {
        int coloring[nodes];
        for (Node n : copy) {
            n.color = rand() % k;
        }
        population.push_back(copy)
    }
    return population;
}

/*
*   TODO
*   Calculate the number of conflicts in a given coloring
*/
int conflicts(Graph<Node> graph) {

}


/*
*   TODO
*   The main algorithm
*/
int algorithm(Graph graph, int n, int t, int lambda, int gamma) {
    int k = max_degree(graph);
    bool foundColoring;
    do {
        foundColoring = false;
        auto pop = populate(graph, n, k);
        for (int _t = 0; _t < t; _t++) {
            graph
        }

    } while (foundColoring);
    return k;
}


int main() {
    auto graph = read_graph("1-Fullins_3.col");
    int n;
    int t;
    int lambda;
    int gamma;
    int best = algorithm(graph, n, lambda, gamma);
    std::cout << best;
}