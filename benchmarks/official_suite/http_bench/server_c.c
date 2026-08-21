#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <stdint.h>
#include <winsock2.h>
#include <windows.h>

#pragma comment(lib, "ws2_32.lib")

static uint64_t xorshift_compute(uint64_t iterations) {
    uint64_t state = 0x853c49e6748fea9bULL;
    for (uint64_t i = 0; i < iterations; i++) {
        state ^= (state << 13);
        state ^= (state >> 7);
        state ^= (state << 17);
        state = state * 6364136223846793005ULL + 1442695040888963407ULL;
    }
    return state;
}

static void send_response(SOCKET sock, const char* status, const char* body) {
    char header[512];
    int body_len = (int)strlen(body);
    int hlen = sprintf(header,
        "HTTP/1.1 %s\r\n"
        "Content-Type: application/json\r\n"
        "Connection: close\r\n"
        "Content-Length: %d\r\n\r\n",
        status, body_len);
    send(sock, header, hlen, 0);
    send(sock, body, body_len, 0);
}

static void handle_health(SOCKET sock) {
    send_response(sock, "200 OK", "{\"status\":\"ok\",\"lang\":\"C (GCC 15.2)\"}");
}

static void handle_compute(SOCKET sock, const char* path) {
    uint64_t n = 1000000;
    const char* q = strstr(path, "n=");
    if (q) n = strtoull(q + 2, NULL, 10);

    LARGE_INTEGER freq, t0, t1;
    QueryPerformanceFrequency(&freq);
    QueryPerformanceCounter(&t0);
    uint64_t hash = xorshift_compute(n);
    QueryPerformanceCounter(&t1);
    int64_t time_us = (t1.QuadPart - t0.QuadPart) * 1000000 / freq.QuadPart;

    char body[256];
    sprintf(body, "{\"hash\":%llu,\"time_us\":%lld,\"lang\":\"C (GCC 15.2)\"}", 
            (unsigned long long)hash, (long long)time_us);
    send_response(sock, "200 OK", body);
}

static void handle_json(SOCKET sock) {
    const char* body =
        "{\"server\":\"C HTTP Backend\",\"version\":\"GCC 15.2\","
        "\"users\":["
        "{\"id\":1,\"name\":\"Alice\",\"score\":9850,\"active\":true},"
        "{\"id\":2,\"name\":\"Bob\",\"score\":8720,\"active\":true},"
        "{\"id\":3,\"name\":\"Charlie\",\"score\":7630,\"active\":false},"
        "{\"id\":4,\"name\":\"Diana\",\"score\":9210,\"active\":true},"
        "{\"id\":5,\"name\":\"Eve\",\"score\":8890,\"active\":true}],"
        "\"metadata\":{\"total_users\":5,\"avg_score\":8860,"
        "\"active_count\":4,\"server_uptime\":99.97}}";
    send_response(sock, "200 OK", body);
}

int main(void) {
    WSADATA wsa;
    WSAStartup(MAKEWORD(2, 2), &wsa);

    SOCKET server_sock = socket(AF_INET, SOCK_STREAM, IPPROTO_TCP);
    int opt = 1;
    setsockopt(server_sock, SOL_SOCKET, SO_REUSEADDR, (char*)&opt, sizeof(opt));

    struct sockaddr_in addr;
    memset(&addr, 0, sizeof(addr));
    addr.sin_family = AF_INET;
    addr.sin_addr.s_addr = INADDR_ANY;
    addr.sin_port = htons(9001);
    bind(server_sock, (struct sockaddr*)&addr, sizeof(addr));
    listen(server_sock, 128);

    printf("[C] HTTP Server listening on :9001\n");
    fflush(stdout);

    while (1) {
        struct sockaddr_in client_addr;
        int client_len = sizeof(client_addr);
        SOCKET client = accept(server_sock, (struct sockaddr*)&client_addr, &client_len);
        if (client == INVALID_SOCKET) continue;

        char buf[4096];
        int n = recv(client, buf, sizeof(buf) - 1, 0);
        if (n <= 0) { closesocket(client); continue; }
        buf[n] = 0;

        char method[16], path[512];
        sscanf(buf, "%15s %511s", method, path);

        if (strstr(path, "/health"))       handle_health(client);
        else if (strstr(path, "/compute")) handle_compute(client, path);
        else if (strstr(path, "/json"))    handle_json(client);
        else send_response(client, "404 Not Found", "{\"error\":\"not found\"}");

        closesocket(client);
    }

    closesocket(server_sock);
    WSACleanup();
    return 0;
}
