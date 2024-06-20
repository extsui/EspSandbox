#include "Util.h"
#include "Volume.h"

// analogRead の範囲が 0~4095 の場合に定義
#define VOLUME_CONFIG_RANGE_4096

namespace {

// サンプリング周期 [ms]
constexpr const uint32_t SamplingIntervalMilliSeconds = 10;

// Arduino の analogRead() の範囲の数 (0~1023)
constexpr const uint32_t AnalogReadRange = 1024;

}

Volume::Volume()
    : m_PinNumber(-1)
    , m_IsInitialized(false)
    , m_NextSampleTime(0)
    , m_pFilter(nullptr)
{
}

/**
 * @param [in] pinNumber ピン番号
 * @param [in] pFilter アナログ生値加工用のフィルタ実体
 * @param [in] level 区分け数 (GetLevel() で取得する時の値域)
 */
void Volume::Initialize(int pinNumber, RcFilter *pFilter, uint32_t levelRange)
{
    ASSERT(!m_IsInitialized);
    ASSERT(pFilter != nullptr);

    m_PinNumber = pinNumber;
    m_IsInitialized = true;
    m_pFilter = pFilter;
    m_LevelRange = levelRange;

    Update();
}

// レベルではなく analogRead() 値域の値を取得する。
// ※フィルタはかけた後の値
uint32_t Volume::GetValue()
{
    ASSERT(m_IsInitialized);
    return m_CurrentOutput;
}

// 初期化時に指定されたレベルの値域内の値で取得する。
uint32_t Volume::GetLevel()
{
    ASSERT(m_IsInitialized);
    // int 演算なので精度向上のため順序を入れ替えている。
    // やりたいことは、y = (<AD 値> / <AD 最大値>) * <取得したい値域>
    return m_CurrentOutput * m_LevelRange / AnalogReadRange;
}

uint32_t Volume::GetValueCorrectedCurveAtoB()
{
    ASSERT(m_IsInitialized);

    uint32_t x = m_CurrentOutput;
    uint32_t y = 0;
    
    // 以下の点を通る一次関数の逆関数をそれぞれの領域で適用して補正する
    // - (0,   0  ), (230,  30  )
    // - (230, 30 ), (680,  270 )
    // - (680, 270), (930,  970 )
    // - (930, 970), (1020, 1020)
    // x 座標系：回転角 (0 ~ 1023 に拡張)
    // y 座標系：AD 値 (0 ~ 1023、A カーブ特性)
    // ※これらの点は ALPS 可変抵抗のデータシートの A カーブ特性のグラフから算出。
    //   ALPS 以外の可変抵抗を使う場合は補正式を変える必要があることに注意。
    if (x < 15)
    {
        // 音量制御用途がメインなので最小付近の領域はゼロに落としたい。
        // RC フィルタをかけた後だと最小付近がゼロまで落ちないので対策。
        y = 0;
    }
    else if (x < 30)
    {
        y = x * 23 / 3;
    }
    else if (x < 270)
    {
        y = x * 15 / 8 + 695 / 4;
    }
    else if (x < 970)
    {
        y = x * 5 / 14 + 4085 / 7;
    }
    else
    {
        y = x * 9 / 5 - 816;
    }
    return y;
}

uint32_t Volume::GetLevelCorrectedCurveAtoB()
{
    ASSERT(m_IsInitialized);
    // int 演算なので精度向上のため順序を入れ替えている。
    // やりたいことは、y = (<AD 値> / <AD 最大値>) * <取得したい値域>
    return GetValueCorrectedCurveAtoB() * m_LevelRange / AnalogReadRange;
}

void Volume::Update()
{
    ASSERT(m_IsInitialized);
    ASSERT(m_pFilter != nullptr);

    uint32_t now = millis();
    if (now <= m_NextSampleTime)
    {
        return;
    }
    m_NextSampleTime += SamplingIntervalMilliSeconds;

    // SeeeduinoXIAO の analogRead() は約 16us かかる
    uint32_t currentInput = analogRead(m_PinNumber);

#ifdef VOLUME_CONFIG_RANGE_4096
    currentInput /= 4;
#endif

    m_CurrentOutput = m_pFilter->Generate(currentInput);
}
