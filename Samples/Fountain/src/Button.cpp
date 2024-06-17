#include "Util.h"
#include "Button.h"

#if BUTTON_CONFIG_USE_INTERRUPT
    #define BUTTON_NO_INTERRUPT()   noInterrupts()
    #define BUTTON_INTERRUPT()      interrupts()
#else
    #define BUTTON_NO_INTERRUPT()
    #define BUTTON_INTERRUPT()
#endif

namespace {

// サンプリング周期 [ms]
static constexpr uint32_t SamplingIntervalMilliSeconds = 10;

// ダブルクリック判定時間 [ms]
// 単クリックが成立してから2回目の単クリックが完了するまでの時間
static constexpr uint32_t DoubleClickThreshold = 300;

// 長クリック判定時間 [ms]
// 押下開始から離しまでの時間
static constexpr uint32_t LongClickThreshold = 500;

// - 引数 value が false なら false を返す
// - 引数 value が true なら false にして true を返す
// BUTTON_CONFIG_USE_INTERRUPT が有効の場合は割り込み排他が行われる。
// BUTTON_CONFIG_USE_INTERRUPT が無効の場合は排他されないため注意。
static bool TestAndReset(bool &value)
{
    BUTTON_NO_INTERRUPT();
    bool retval = false;
    if (value)
    {
        value = false;
        retval = true;
    }
    BUTTON_INTERRUPT();
    return retval;
}

}

Button::Button()
    : m_PinNumber(-1)
    , m_IsInitialized(false)
    , m_IsPositive(false)
    , m_NextSampleTime(0)
    , m_State(false)
    , m_PressStartTime(0)
    , m_LastSingleClickedTime(0)
    , m_WasSingleClicked(false)
    , m_WasLongClicked(false)
    , m_WasDoubleClicked(false)
{
    memset(m_History, 0, sizeof(m_History));
}

void Button::Initialize(int pinNumber, bool isPositive, bool isPullUp)
{
    ASSERT(!m_IsInitialized);

    m_PinNumber = pinNumber;
    m_IsInitialized = true;
    m_IsPositive = isPositive;

    if (isPullUp)
    {
        pinMode(pinNumber, INPUT_PULLUP);
    }

    Update();
}

/**
 * 今の端子の状態を設定を加味した上で取得する
 * @retval true: 押下
 * @retval false: 離し
 * @note チャタリング除去前の値であることに注意
 */
bool Button::Read()
{
    ASSERT(m_IsInitialized);

    int pinState = digitalRead(m_PinNumber);
    if ((m_IsPositive && pinState == HIGH) ||
        (!m_IsPositive && pinState == LOW))
    {
        return true;
    }
    else
    {
        return false;
    }
}

bool Button::WasSingleClicked()
{
    ASSERT(m_IsInitialized);
    return TestAndReset(m_WasSingleClicked);
}

bool Button::WasLongClicked()
{
    ASSERT(m_IsInitialized);
    return TestAndReset(m_WasLongClicked);
}

bool Button::WasDoubleClicked()
{
    ASSERT(m_IsInitialized);
    return TestAndReset(m_WasDoubleClicked);
}

void Button::ClearEvents()
{
    ASSERT(m_IsInitialized);
    BUTTON_NO_INTERRUPT();
    m_WasSingleClicked = false;
    m_WasLongClicked = false;
    m_WasDoubleClicked = false;
    BUTTON_INTERRUPT();
}

void Button::Update()
{
    ASSERT(m_IsInitialized);

    uint32_t now = millis();
    if (now <= m_NextSampleTime)
    {
        return;
    }
    m_NextSampleTime += SamplingIntervalMilliSeconds;

    // サンプリングタイミング成立
    m_History[2] = m_History[1];
    m_History[1] = m_History[0];
    m_History[0] = Read();

    // 二回一致で確定
    bool isStateChanged = false;
    if ((m_History[0] == true) && (m_History[1] == true) && (m_History[2] == false))
    {
        // 押下状態へ移行
        if (m_State == false)
        {
            isStateChanged = true;
        }
        m_State = true;
    }
    else if ((m_History[0] == false) && (m_History[1] == false) && (m_History[2] == true))
    {
        // 離し状態へ移行
        if (m_State == true)
        {
            isStateChanged = true;
        }
        m_State = false;
    }
    else
    {
        // NOP (チャタリング中)
    }

    if ((m_State == true) && (isStateChanged == true))
    {
        // 押下開始
        m_PressStartTime = now;
    }
    else if ((m_State == false) && (isStateChanged == true))
    {
        // 離した瞬間
        // 判定優先度: ダブルクリック > 長クリック > クリック
        if (now - m_LastSingleClickedTime < DoubleClickThreshold)
        {
            m_WasDoubleClicked = true;
        }
        else if (now - m_PressStartTime >= LongClickThreshold)
        {
            m_WasLongClicked = true;
        }
        else
        {
            m_WasSingleClicked = true;
            m_LastSingleClickedTime = now;
        }
    }
}
