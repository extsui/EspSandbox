#ifndef VOLUME_H
#define VOLUME_H

#include "Util.h"

/**
 * 一次 RC フィルタ (ローパスフィルタ)
 * 
 * - 伝達関数: y = y[N-1] * r + x[N] * (1 - r)
 * - CortexM0+ では浮動小数演算器がないので
 *   伝達関数の比率 0.01~0.99 を 1~99 として扱う
 * - 値を大きくするほど滑らかになるが応答が遅くなる
 */
class RcFilter
{
public:
    explicit RcFilter(uint32_t rate)
        : m_PreviousOutput(0)
        , m_Rate(rate)
    {
        ASSERT(rate < 100);
    }

    uint32_t Generate(uint32_t currentInput)
    {
        uint32_t currentOutput =
            (m_PreviousOutput *        m_Rate  / 100) +
            (currentInput     * (100 - m_Rate) / 100);
        m_PreviousOutput = currentOutput;
        return currentOutput;
    }

private:
    uint32_t m_PreviousOutput;
    uint32_t m_Rate;
};

/**
 * Arudino 用可変抵抗管理クラス
 * 
 * - 可変抵抗を対象にしているが A/B/C カーブは考慮しない。
 * - 簡易フィルタ付き。
 */
class Volume
{
public:
    Volume();
    void Initialize(int pinNumber, RcFilter *pFilter, uint32_t levelRange);
    uint32_t GetValue();
    uint32_t GetLevel();
    uint32_t GetValueCorrectedCurveAtoB();
    uint32_t GetLevelCorrectedCurveAtoB();
    void Update();

private:
    // 設定
    int m_PinNumber;
    bool m_IsInitialized;

    uint32_t m_NextSampleTime;
    uint32_t m_CurrentOutput;

    RcFilter *m_pFilter;
    uint32_t m_LevelRange;
};

#endif /* VOLUME_H */
