// Ultra-Heavy Benchmark 4: 5,000,000 Node Knowledge Graph & Binary Tree Traversal
// Cache-Locality & Pointer-Chasing Stress Test
#include <stdio.h>
#include <stdlib.h>
#include <stdint.h>
#include <windows.h>

#define TREE_DEPTH 22 // 2^22 - 1 = 4,194,303 Nodes

typedef struct TreeNode {
    int64_t value;
    struct TreeNode* left;
    struct TreeNode* right;
} TreeNode;

// Contiguous Memory Arena Pool (End Deterministic Region Layout)
typedef struct {
    TreeNode* pool;
    size_t count;
} NodePool;

static TreeNode* create_tree_region(NodePool* np, int depth, int64_t val) {
    if (depth <= 0) return NULL;
    TreeNode* node = &np->pool[np->count++];
    node->value = val;
    node->left = create_tree_region(np, depth - 1, val * 2 + 1);
    node->right = create_tree_region(np, depth - 1, val * 2 + 2);
    return node;
}

static int64_t sum_tree(TreeNode* root) {
    if (!root) return 0;
    return root->value + sum_tree(root->left) + sum_tree(root->right);
}

int main(void) {
    LARGE_INTEGER freq, start, end;
    QueryPerformanceFrequency(&freq);

    size_t total_nodes = (1ULL << TREE_DEPTH) - 1;

    printf("========================================================================\n");
    printf("🌳 ULTRA-HEAVY TEST 4: %zu Nodes Graph & Tree Cache-Locality Stress Test\n", total_nodes);
    printf("========================================================================\n");

    NodePool np;
    np.pool = (TreeNode*)malloc(total_nodes * sizeof(TreeNode));
    np.count = 0;

    printf("Building %zu-node Knowledge Graph in Contiguous End Region Pool...\n", total_nodes);
    QueryPerformanceCounter(&start);
    TreeNode* root = create_tree_region(&np, TREE_DEPTH, 1);
    QueryPerformanceCounter(&end);
    double build_ms = (double)(end.QuadPart - start.QuadPart) * 1000.0 / freq.QuadPart;

    printf("Traversing & Summing %zu Nodes across Memory...\n", total_nodes);
    QueryPerformanceCounter(&start);
    int64_t total_sum = sum_tree(root);
    QueryPerformanceCounter(&end);
    double traverse_ms = (double)(end.QuadPart - start.QuadPart) * 1000.0 / freq.QuadPart;

    printf("\n📊 GRAPH TRAVERSAL & CACHE-LOCALITY RESULTS:\n");
    printf("  - Total Tree Nodes:      %zu nodes (depth %d)\n", total_nodes, TREE_DEPTH);
    printf("  - Total Traversal Sum:   %lld\n", (long long)total_sum);
    printf("  - Tree Allocation Time:  %.2f ms (%.2fM nodes/sec)\n", build_ms, (total_nodes / build_ms) / 1000.0);
    printf("  - Graph Traversal Time:  %.2f ms (%.2fM nodes traversed/sec)\n", traverse_ms, (total_nodes / traverse_ms) / 1000.0);
    printf("  - Memory Footprint:      %.2f MB\n", (total_nodes * sizeof(TreeNode)) / (1024.0 * 1024.0));

    free(np.pool);
    return 0;
}
