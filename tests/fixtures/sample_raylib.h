// Sample C Library Header for Raylib / Game Engine
#ifndef SAMPLE_RAYLIB_H
#define SAMPLE_RAYLIB_H

#define RAYLIB_VERSION "5.0.0"
#define MAX_TOUCH_POINTS 10
#define FLAG_WINDOW_RESIZABLE 4

typedef enum {
    LOG_ALL = 0,
    LOG_TRACE = 1,
    LOG_DEBUG = 2,
    LOG_INFO = 3,
    LOG_WARNING = 4,
    LOG_ERROR = 5
} TraceLogLevel;

typedef struct {
    float x;
    float y;
} Vector2;

typedef struct {
    unsigned char r;
    unsigned char g;
    unsigned char b;
    unsigned char a;
} Color;

// Core Window and Drawing Functions
extern void InitWindow(int width, int height, const char *title);
extern bool WindowShouldClose(void);
extern void CloseWindow(void);
extern void BeginDrawing(void);
extern void EndDrawing(void);
extern void ClearBackground(Color color);
extern void DrawRectangle(int posX, int posY, int width, int height, Color color);
extern void DrawCircle(int centerX, int centerY, float radius, Color color);
extern double GetTime(void);
extern int GetFPS(void);

#endif // SAMPLE_RAYLIB_H
