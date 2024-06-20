#include "Util.h"

void DumpBackTrace() noexcept
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

void HexDump(const uint8_t data[], size_t length) noexcept
{
    for (int i = 0; i < length; i++)
    {
        LOG("%02x ", data[i]);
        // 16 個表示毎に改行、ただし先頭行と最終データが丁度 16 の倍数の場合は省略
        if (((i + 1) % 16 == 0) && (i > 0) && ((i + 1) != length))
        {
            LOG("\n");
        }
    }
    LOG("\n");
}
