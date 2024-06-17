#include <Arduino.h>
#include <stdint.h>
#include "Ht16k33.h"
#include "Building.h"
#include "Util.h"

namespace {

// 第一次元：7 セグ横方向の 6 桁分
// 第二次元：7 セグ縦方向の 6 ユニット分
// 第三次元：メタ的な 7 セグのセグメント (a, b, ..., *)
constexpr unsigned char MetaSegmentTable[8][6][6] =
{
    // a
    {
        { 0x00, 0x21, 0x3B, 0x3B, 0x3B, 0x00, },
        { 0x00, 0x00, 0x80, 0x80, 0x80, 0x00, },
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, },
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, },
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, },
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, },
    },
    // b
    {
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, },
        { 0x00, 0x00, 0x00, 0x00, 0x71, 0x1E, },
        { 0x00, 0x00, 0x00, 0x00, 0xE2, 0x8E, },
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, },
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, },
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, },
    },
    // c
    {
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, },
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, },
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, },
        { 0x00, 0x00, 0x00, 0x00, 0x31, 0x1A, },
        { 0x00, 0x00, 0x00, 0x00, 0xE3, 0x8E, },
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, },
    },
    // d
    {
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, },
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, },
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, },
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, },
        { 0x00, 0x01, 0x11, 0x11, 0x00, 0x00, },
        { 0x00, 0xC0, 0xC6, 0xC6, 0x04, 0x00, },
    },
    // e
    {
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, },
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, },
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, },
        { 0x21, 0x1A, 0x00, 0x00, 0x00, 0x00, },
        { 0xF3, 0x8E, 0x00, 0x00, 0x00, 0x00, },
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, },
    },
    // f
    {
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, },
        { 0x21, 0xFF, 0x00, 0x00, 0x00, 0x00, },
        { 0x61, 0xCE, 0x00, 0x00, 0x00, 0x00, },
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, },
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, },
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, },
    },
    // g
    {
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, },
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, },
        { 0x00, 0x01, 0x11, 0x11, 0x00, 0x00, },
        { 0x00, 0x40, 0xC6, 0xC6, 0x84, 0x00, },
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, },
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, },
    },
    // *
    {
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, },
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, },
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, },
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, },
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, },
        { 0x00, 0x00, 0x00, 0x00, 0x00, 0xC6, },
    },
};

}

void Building::Initialize(TwoWire* pWire) noexcept
{
    ASSERT(m_pWire == nullptr);
    m_pWire = pWire;
    
    for (int unit = 0; unit < UnitCount; unit++)
    {
        Ht16k33::Initialize(m_pWire, Ht16k33::BaseAddress + unit);
        Ht16k33::SetBrightness(m_pWire, Ht16k33::BaseAddress + unit, 4);
    }
}

void Building::SetBrightness(uint8_t brightness) noexcept
{
    ASSERT(m_pWire != nullptr);
    if (brightness >= 16)
    {
        LOG("brightness range error! (%d)\n", brightness);
        return;
    }
    for (int unit = 0; unit < UnitCount; unit++)
    {
        Ht16k33::SetBrightness(m_pWire, Ht16k33::BaseAddress + unit, brightness);
    }
}

void Building::Clear() noexcept
{
    ASSERT(m_pWire != nullptr);
    memset(m_Display, 0, sizeof(m_Display));
}

void Building::Fill() noexcept
{
    ASSERT(m_pWire != nullptr);
    memset(m_Display, 0xFF, sizeof(m_Display));
}

void Building::SetPattern(int x, int y, uint8_t pattern) noexcept
{
    ASSERT(m_pWire != nullptr);
    ASSERT(0 <= x && x < DigitX);
    ASSERT(0 <= y && y < DigitY);
    m_Display[y][x] = pattern;
}

void Building::OrPattern(int x, int y, uint8_t pattern) noexcept
{
    ASSERT(m_pWire != nullptr);
    ASSERT(0 <= x && x < DigitX);
    ASSERT(0 <= y && y < DigitY);
    m_Display[y][x] |= pattern;
}

void Building::AndPattern(int x, int y, uint8_t pattern) noexcept
{
    ASSERT(m_pWire != nullptr);
    ASSERT(0 <= x && x < DigitX);
    ASSERT(0 <= y && y < DigitY);
    m_Display[y][x] &= ~pattern;
}

void Building::Update() noexcept
{
    ASSERT(m_pWire != nullptr);
/*
    LOG("Pattern:\n");
    for (int y = 0; y < DigitY; y++)
    {
        for (int x = 0; x < DigitX; x++)
        {
            LOG("%02x ", m_Display[y][x]);
        }
        LOG("\n");
    }
    LOG("\n\n");
*/
    for (int unit = 0; unit < UnitCount; unit++)
    {
        // HT16K33 1個に接続されている 7 セグは 12 個だが、表示用の
        // RAM の構成が横分割ではなく縦分割なので 16Byte 全域を使う必要がある
        uint8_t display[DigitCount];
        memset(display, 0, sizeof(display));
        for (int y = 0; y < 2; y++)
        {
            for (int x = 0; x < DigitX; x++)
            {
                CreateDisplayData(display, (y * DigitX) + x, m_Display[(unit * 2) + y][x]);
            }
        }
/*
        // DEBUG:
        for (int i = 0; i < DigitCount; i++)
        {
            LOG("%02x ", display[i]);
        }
        LOG("\n");
*/
        Ht16k33::SetDisplay(m_pWire, Ht16k33::BaseAddress + unit, display, DigitCount);

    }
}

void Building::CreateDisplayData(uint8_t* pDisplay, uint8_t digit, uint8_t pattern) noexcept
{
    /*
        *         HT16K33 の RAM 構成
        * -----+----------------+---------------
        *      | ROW0  1  ... 7 | 8  9  ...  15 
        * -----+----------------+---------------
        * COL0 |      00h       |       01h
        *    1 |      02h       |       03h
        *    : |       :        |        :
        * COL7 |      0Eh       |       0Fh
        * -----+----------------+---------------
        * 
        *    RAM の構成に対する 7 セグの割り当て
        * -----+-------------+--------------------
        *  C\R | 0  1  ... 7 | 8 ... 11 12 ... 15
        * -----+-------------+--------------------
        *    0 | a  a  ... a | a ... a  x  ... x
        *    1 | b  b  ... b | b ... b  x  ... x
        *    2 | c  c  ... c | c ... c  x  ... x
        *    3 | d  d  ... d | d ... d  x  ... x
        *    4 | e  e  ... e | e ... e  x  ... x
        *    5 | f  f  ... f | f ... f  x  ... x
        *    6 | g  g  ... g | g ... g  x  ... x
        *    7 | *  *  ... * | * ... *  x  ... x
        * -----+-------------+--------------------
        *        ^  ^          ^        ^^^^^^^^ 
        *        |  2桁目      |        (4桁分未使用)
        *        1桁目         8桁目
        */
    ASSERT(digit < 12);
    if (digit < 8)
    {
        pDisplay[0]  |= ((pattern & (1 << 7)) != 0) ? (1 << digit) : 0; // a
        pDisplay[2]  |= ((pattern & (1 << 6)) != 0) ? (1 << digit) : 0; // b
        pDisplay[4]  |= ((pattern & (1 << 5)) != 0) ? (1 << digit) : 0; // c
        pDisplay[6]  |= ((pattern & (1 << 4)) != 0) ? (1 << digit) : 0; // d
        pDisplay[8]  |= ((pattern & (1 << 3)) != 0) ? (1 << digit) : 0; // e
        pDisplay[10] |= ((pattern & (1 << 2)) != 0) ? (1 << digit) : 0; // f
        pDisplay[12] |= ((pattern & (1 << 1)) != 0) ? (1 << digit) : 0; // g
        pDisplay[14] |= ((pattern & (1 << 0)) != 0) ? (1 << digit) : 0; // *
    }
    else
    {
        pDisplay[1]  |= ((pattern & (1 << 7)) != 0) ? (1 << (digit - 8)) : 0; // a
        pDisplay[3]  |= ((pattern & (1 << 6)) != 0) ? (1 << (digit - 8)) : 0; // b
        pDisplay[5]  |= ((pattern & (1 << 5)) != 0) ? (1 << (digit - 8)) : 0; // c
        pDisplay[7]  |= ((pattern & (1 << 4)) != 0) ? (1 << (digit - 8)) : 0; // d
        pDisplay[9]  |= ((pattern & (1 << 3)) != 0) ? (1 << (digit - 8)) : 0; // e
        pDisplay[11] |= ((pattern & (1 << 2)) != 0) ? (1 << (digit - 8)) : 0; // f
        pDisplay[13] |= ((pattern & (1 << 1)) != 0) ? (1 << (digit - 8)) : 0; // g
        pDisplay[15] |= ((pattern & (1 << 0)) != 0) ? (1 << (digit - 8)) : 0; // *
    }
}

constexpr uint8_t NumberSegmentTable[] =
{
    0xFC,   // 0
    0x60,   // 1
    0xDA,   // 2
    0xF2,   // 3
    0x66,   // 4
    0xB6,   // 5
    0xBE,   // 6
    0xE4,   // 7
    0xFE,   // 8
    0xF6,   // 9
    0xEE,   // A
    0x3E,   // b
    0x1A,   // c
    0x7A,   // d
    0x9E,   // E
    0x8E,   // F
};
