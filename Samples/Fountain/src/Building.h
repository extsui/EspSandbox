#ifndef BUILDING_H
#define BUILDING_H

#include <Arduino.h>
#include <stdint.h>

class Building
{
public:
    static constexpr int DigitX = 6;
    static constexpr int DigitY = 6;

    // HT16K33 の数
    static constexpr int UnitCount = 3;
    // HT16K33 1個あたりの管理桁数
    static constexpr int DigitCount = 16;
    
public:
    void Initialize(TwoWire* pWire) noexcept;
    void SetBrightness(uint8_t brightness) noexcept;
    void Clear() noexcept;
    void Fill() noexcept;
    void SetPattern(int x, int y, uint8_t pattern) noexcept;
    void OrPattern(int x, int y, uint8_t pattern) noexcept;
    void AndPattern(int x, int y, uint8_t pattern) noexcept;
    void Update() noexcept;

private:
    static void CreateDisplayData(uint8_t* pDisplay, uint8_t digit, uint8_t pattern) noexcept;

private:
    uint8_t m_Display[DigitY][DigitX];
    TwoWire* m_pWire;
};

#endif // BUILDING_H