#ifndef HT16K33_H
#define HT16K33_H

#include <stdint.h>
#include <Wire.h>

class Ht16k33
{
public:
	// ベースアドレス (Base + [ 0x00 : 0x07 ])
	static constexpr uint8_t BaseAddress = 0x70;

	// HT16K33 で規定されている輝度
	static constexpr uint8_t BrightnessMin = 0;
	static constexpr uint8_t BrightnessMax = 15;

    static int Initialize(TwoWire *i2c, uint8_t addr);
    static int SetBrightness(TwoWire *i2c, uint8_t addr, uint8_t brightness);
    static int SetDisplay(TwoWire *i2c, uint8_t addr, const uint8_t data[], uint8_t length);
};

#endif /* HT16K33_H */
