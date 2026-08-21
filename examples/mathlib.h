/* End Language Generated C Header File */
#pragma once

#include <stdint.h>
#include <stdbool.h>

#if defined(_WIN32) || defined(__CYGWIN__)
    #ifdef BUILDING_END_DLL
        #define END_API __declspec(dllexport)
    #else
        #define END_API __declspec(dllimport)
    #endif
#elif defined(__GNUC__) || defined(__clang__)
    #define END_API __attribute__((visibility("default")))
#else
    #define END_API
#endif

#ifdef __cplusplus
extern "C" {
#endif

typedef struct CalcResult {
    int64_t sum;
    int64_t product;
    bool is_valid;
} CalcResult;

END_API int64_t end_add(int64_t a, int64_t b);
END_API int64_t end_multiply(int64_t a, int64_t b);
END_API int64_t end_compute_hash(uint64_t id, int32_t iterations);
END_API int64_t end_process_batch(int32_t count);

#ifdef __cplusplus
}
#endif
