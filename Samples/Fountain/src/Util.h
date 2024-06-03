#ifndef UTIL_H
#define UTIL_H

#include <Arduino.h>

#define LOG(...) Serial.printf(__VA_ARGS__)

// TODO: cpp ファイルに移動すること
void DumpBackTrace()
{
    constexpr int BackTraceDepthMax = 10;
    void *fp = __builtin_frame_address(0);

    LOG("Backtrace:\n");
    for (int depth = 0; depth < BackTraceDepthMax; depth++)
    {
        if (fp == nullptr)
        {
            break;
        }

        void *pc = *((void**)fp - 1);
        LOG("#%d %p\n", depth, pc);

        uint32_t* addr = reinterpret_cast<uint32_t*>(fp);
        addr -= 2;
        fp = (void*)(*(uint32_t*)addr);
    }
}

static inline void ABORT_NO_MESSAGE() { DumpBackTrace(); while (1); }

#define ABORT() \
    LOG("Abort: file %s, line %d\n", __FILE__, __LINE__); \
    ABORT_NO_MESSAGE()

#define __ASSERT(expr, file, line)                  \
	LOG("Assertion failed: %s, file %s, line %d\n", \
		expr, file, line),                          \
	ABORT_NO_MESSAGE()

#define ASSERT(expr)                              \
    ((expr) ? ((void)0) :                         \
    (void)(__ASSERT(#expr, __FILE__, __LINE__)))

// [[maybe_unused]] 属性 (C++17) を使えるようだが
// 非常に見辛くなるのでマクロで代用する
#define UNUSED(var) ((void)var)

#endif /* UTIL_H */
