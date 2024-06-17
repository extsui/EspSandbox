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
