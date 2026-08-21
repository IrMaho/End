#include <stdio.h>
#include <stdint.h>

int main() {
    uint64_t id = 100;
    
    // Naive loop with uint64_t wrapping
    uint64_t hash = 17;
    for (uint64_t j = 0; j < 50; j++) {
        hash = (hash * 31) + id + j;
    }
    printf("Naive loop for id=100: %lld\n", (long long)hash);

    // Closed form coefficients
    uint64_t c_hash = 17;
    uint64_t c_id = 0;
    uint64_t c_const = 0;
    for (uint64_t j = 0; j < 50; j++) {
        c_hash = c_hash * 31;
        c_id = (c_id * 31) + 1;
        c_const = (c_const * 31) + j;
    }

    uint64_t closed_hash = (c_hash) + (id * c_id) + c_const;
    printf("Closed form for id=100: %lld\n", (long long)closed_hash);
    printf("Match: %s\n", (hash == closed_hash) ? "TRUE" : "FALSE");
    printf("c_id multiplier: %lluULL, c_base_const: %lluULL\n", (unsigned long long)c_id, (unsigned long long)(c_hash + c_const));

    return 0;
}
