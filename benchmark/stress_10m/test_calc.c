#include <stdio.h>
#include <stdint.h>

int main() {
    uint64_t c_hash = 17;
    uint64_t c_id = 0;
    uint64_t c_const = 0;
    for (uint64_t j = 0; j < 32; j++) {
        c_hash = c_hash * 31;
        c_id = (c_id * 31) + 1;
        c_const = (c_const * 31) + j;
    }
    printf("const C_MUL = %lluULL;\n", (unsigned long long)c_id);
    printf("const C_BASE = %lluULL;\n", (unsigned long long)(c_hash + c_const));

    // Verify
    uint64_t id = 12345;
    uint64_t hash = 17;
    for (uint64_t j = 0; j < 32; j++) {
        hash = (hash * 31) + id + j;
    }
    uint64_t fast = (id * c_id) + (c_hash + c_const);
    printf("Naive: %lld, Fast: %lld, Match: %s\n", (long long)hash, (long long)fast, (hash == fast) ? "TRUE" : "FALSE");
    return 0;
}
