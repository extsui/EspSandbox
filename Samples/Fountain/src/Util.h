#ifndef UTIL_H
#define UTIL_H

#include <Arduino.h>

#define LOG(...) Serial.printf(__VA_ARGS__)

void DumpBackTrace() noexcept;

static inline void ABORT_NO_MESSAGE() noexcept
{
    DumpBackTrace();
    while (1);
}

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

void HexDump(const uint8_t data[], size_t length) noexcept;

#endif /* UTIL_H */
