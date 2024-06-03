#include <Arduino.h>
#include "Util.h"

void setup()
{
    Serial.begin(115200);

    // USB-CDC のせいか起動直後にログを大量に出しても PC 側に表示されない
    // 適当なディレイを入れると安定するようになったので暫定対処
    delay(1000);
}

void loop()
{
    LOG("Hello World!\n");
    delay(1000);
}
