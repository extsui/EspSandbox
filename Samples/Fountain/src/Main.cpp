#include <Arduino.h>
#include <stdint.h>
#include "Tone.h"
#include "Util.h"

namespace Pin {

constexpr int DipSwitch0 = D2;
constexpr int DipSwitch1 = D3;
constexpr int DipSwitch2 = D4;
constexpr int DipSwitch3 = D5;

constexpr int Buzzer = D6;

}

namespace {

uint8_t g_OwnAddress = 0xFF;

void InitializeDipSwitch() noexcept
{
    pinMode(Pin::DipSwitch0, INPUT_PULLUP);
    pinMode(Pin::DipSwitch1, INPUT_PULLUP);
    pinMode(Pin::DipSwitch2, INPUT_PULLUP);
    pinMode(Pin::DipSwitch3, INPUT_PULLUP);
}

// TODO: チャタリング除去
// TODO: 動的アドレス変更対応
uint8_t ReadOwnAddress() noexcept
{
    // 全て負論理
    uint8_t d0 = ~digitalRead(Pin::DipSwitch0) & 0x1;
    uint8_t d1 = ~digitalRead(Pin::DipSwitch1) & 0x1;
    uint8_t d2 = ~digitalRead(Pin::DipSwitch2) & 0x1;
    uint8_t d3 = ~digitalRead(Pin::DipSwitch3) & 0x1;
    return ((d3 << 3) | (d2 << 2) | (d1 << 1) | (d0));
}

void InitializeBuzzer() noexcept
{
    pinMode(Pin::Buzzer, OUTPUT);
}

// TODO: メロディスレッドに分離
void PlayStartupMelody() noexcept
{
    tone(Pin::Buzzer, Note::C6, 100);
    tone(Pin::Buzzer, Note::E6, 100);
    tone(Pin::Buzzer, Note::G6, 100);
}

}

void setup()
{
    Serial.begin(115200);

    InitializeDipSwitch();
    g_OwnAddress = ReadOwnAddress();

    InitializeBuzzer();

    // USB-CDC のせいか起動直後にログを大量に出しても PC 側に表示されない
    // 適当なディレイを入れると安定するようになったので暫定対処
    delay(1000);

    LOG("OwnAddress: 0x%02x\n", g_OwnAddress);
    PlayStartupMelody();
}

void loop()
{
    LOG("Hello World!\n");
    delay(1000);
}
