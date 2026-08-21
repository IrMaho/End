const std = @import("std");

const c = @cImport({
    @cInclude("stdio.h");
    @cInclude("stdlib.h");
    @cInclude("string.h");
    @cInclude("winsock2.h");
    @cInclude("windows.h");
});

fn xorshiftCompute(iterations: u64) u64 {
    var state: u64 = 0x853c49e6748fea9b;
    var i: u64 = 0;
    while (i < iterations) : (i += 1) {
        state ^= (state << 13);
        state ^= (state >> 7);
        state ^= (state << 17);
        state = state *% 6364136223846793005 +% 1442695040888963407;
    }
    return state;
}

fn sendResponse(sock: c.SOCKET, status: [*:0]const u8, body: [*:0]const u8) void {
    var header: [512]u8 = undefined;
    const body_len = c.strlen(body);
    const hlen = c.sprintf(&header, "HTTP/1.1 %s\r\nContent-Type: application/json\r\nConnection: close\r\nContent-Length: %d\r\n\r\n", status, @as(c_int, @intCast(body_len)));
    _ = c.send(sock, &header, hlen, 0);
    _ = c.send(sock, body, @as(c_int, @intCast(body_len)), 0);
}

pub fn main() void {
    var wsa: c.WSADATA = undefined;
    _ = c.WSAStartup(c.MAKEWORD(2, 2), &wsa);

    const server_sock = c.socket(c.AF_INET, c.SOCK_STREAM, c.IPPROTO_TCP);
    var opt: c_int = 1;
    _ = c.setsockopt(server_sock, c.SOL_SOCKET, c.SO_REUSEADDR, @as([*]const u8, @ptrCast(&opt)), @sizeOf(c_int));

    var addr: c.struct_sockaddr_in = std.mem.zeroes(c.struct_sockaddr_in);
    addr.sin_family = c.AF_INET;
    addr.sin_addr.S_un.S_addr = c.INADDR_ANY;
    addr.sin_port = c.htons(9002);
    _ = c.bind(server_sock, @as(*const c.struct_sockaddr, @ptrCast(&addr)), @sizeOf(c.struct_sockaddr_in));
    _ = c.listen(server_sock, 128);

    _ = c.printf("[Zig] HTTP Server listening on :9002\n");

    while (true) {
        var client_addr: c.struct_sockaddr_in = undefined;
        var client_len: c_int = @sizeOf(c.struct_sockaddr_in);
        const client_sock = c.accept(server_sock, @as(*c.struct_sockaddr, @ptrCast(&client_addr)), &client_len);
        if (client_sock == c.INVALID_SOCKET) continue;

        var buf: [4096]u8 = undefined;
        const n = c.recv(client_sock, &buf, 4095, 0);
        if (n <= 0) {
            _ = c.closesocket(client_sock);
            continue;
        }
        buf[@as(usize, @intCast(n))] = 0;

        // Parse path
        if (c.strstr(&buf, "/health") != null) {
            sendResponse(client_sock, "200 OK", "{\"status\":\"ok\",\"lang\":\"Zig 0.16.0\"}");
        } else if (c.strstr(&buf, "/compute") != null) {
            var freq: c.LARGE_INTEGER = undefined;
            var t0: c.LARGE_INTEGER = undefined;
            var t1: c.LARGE_INTEGER = undefined;
            _ = c.QueryPerformanceFrequency(&freq);
            _ = c.QueryPerformanceCounter(&t0);
            const hash = xorshiftCompute(1000000);
            _ = c.QueryPerformanceCounter(&t1);
            const time_us = @divTrunc((t1.QuadPart - t0.QuadPart) * 1000000, freq.QuadPart);

            var body: [256]u8 = undefined;
            _ = c.sprintf(&body, "{\"hash\":%llu,\"time_us\":%lld,\"lang\":\"Zig 0.16.0\"}", @as(c_ulonglong, hash), @as(c_longlong, time_us));
            sendResponse(client_sock, "200 OK", @as([*:0]const u8, @ptrCast(&body)));
        } else if (c.strstr(&buf, "/json") != null) {
            sendResponse(client_sock, "200 OK", "{\"server\":\"Zig HTTP Backend\",\"version\":\"0.16.0\",\"users\":[{\"id\":1,\"name\":\"Alice\",\"score\":9850,\"active\":true},{\"id\":2,\"name\":\"Bob\",\"score\":8720,\"active\":true},{\"id\":3,\"name\":\"Charlie\",\"score\":7630,\"active\":false},{\"id\":4,\"name\":\"Diana\",\"score\":9210,\"active\":true},{\"id\":5,\"name\":\"Eve\",\"score\":8890,\"active\":true}],\"metadata\":{\"total_users\":5,\"avg_score\":8860,\"active_count\":4,\"server_uptime\":99.97}}");
        } else {
            sendResponse(client_sock, "404 Not Found", "{\"error\":\"not found\"}");
        }

        _ = c.closesocket(client_sock);
    }
}
