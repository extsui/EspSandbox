#include "Ht16k33.h"

int Ht16k33::Initialize(TwoWire *i2c, uint8_t addr)
{
    // TODO: 通信エラー時の return -1

    // オシレータ ON
    i2c->beginTransmission(addr);
    i2c->write(0x21);
    i2c->endTransmission();

    // 輝度設定 (0)
    i2c->beginTransmission(addr);
    i2c->write(0xE0 | 0);
    i2c->endTransmission();
    
    // 点滅設定 (点滅無し)
    i2c->beginTransmission(addr);
    i2c->write(0x80 | 0x01 | 0);
    i2c->endTransmission();

    return 0;
}

int Ht16k33::SetBrightness(TwoWire *i2c, uint8_t addr, uint8_t brightness)
{
    // TODO: 通信エラー時の return -1

    if (brightness > BrightnessMax)
    {
        return -1;
    }

    // 輝度設定 (0~15)
    i2c->beginTransmission(addr);
    i2c->write(0xE0 | brightness);
    i2c->endTransmission();

    return 0;
}

int Ht16k33::SetDisplay(TwoWire *i2c, uint8_t addr, const uint8_t data[], uint8_t length)
{
    // TODO: 通信エラー時の return -1

    i2c->beginTransmission(addr);
    
    // 表示 RAM (16Byte) は 0 番地から始まるのでそれを指定
    i2c->write(0x00);

    i2c->write(data, length);
    i2c->endTransmission();

    return 0;
}
