#include <Arduino.h>
#include <Wire.h>
#include <stdint.h>
#include "Building.h"
#include "Tone.h"
#include "Util.h"

// TODO: 後で分離
#include <esp_now.h>
#include <WiFi.h>

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

// ESP_NOW マスタ関連
namespace {

void OnDataSendCompleteCallback(const uint8_t *pMac, esp_now_send_status_t status)
{
    char macStr[18];
    snprintf(macStr, sizeof(macStr), "%02x:%02x:%02x:%02x:%02x:%02x",
             pMac[0], pMac[1], pMac[2], pMac[3], pMac[4], pMac[5]);
    LOG("Last Packet Sent to: %s\n", macStr);
    LOG("Last Packet Send Status: %s\n", (status == ESP_NOW_SEND_SUCCESS ? "Delivery Success" : "Delivery Fail"));
}

}

constexpr int EspNowSlaveMax = 15;
esp_now_peer_info_t g_Slaves[EspNowSlaveMax] = {};
esp_now_peer_info_t g_AnySlave = {};
int g_SlaveCount = 0;

constexpr int EspNowChannel = 1;

constexpr uint8_t BroadcastAddress[] = { 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF };

TwoWire& g_Wire = Wire;
Building g_Building;

void InitializeByStandAloneMode() noexcept
{
    g_Wire.begin(D10, D7, 400000);
    g_Building.Initialize(&g_Wire);
    g_Building.Clear();
    g_Building.Update();
}

void setup()
{
    Serial.begin(115200);

    // TODO: DIPSW で分岐
    InitializeByStandAloneMode();

    InitializeDipSwitch();
    g_OwnAddress = ReadOwnAddress();

    InitializeBuzzer();
    PlayStartupMelody();

    // USB-CDC のせいか起動直後にログを大量に出しても PC 側に表示されない
    // 適当なディレイを入れると安定するようになったので暫定対処
    delay(1000);

    LOG("OwnAddress: 0x%02x\n", g_OwnAddress);

    // TODO: 以降は後で分離
    if (g_OwnAddress == 0)
    {
        WiFi.mode(WIFI_STA);
        LOG("Master MAC: %s\n", WiFi.macAddress().c_str());

        WiFi.disconnect();
        if (esp_now_init() == ESP_OK)
        {
            LOG("ESP_NOW Initialize Success\n");
        }
        else
        {
            LOG("ESP_NOW Initialize Failed\n");
            ESP.restart();
        }

        esp_now_register_send_cb(OnDataSendCompleteCallback);

        // ブロードキャストする場合でも peer として登録しておく必要がある
        g_AnySlave.channel = EspNowChannel;
        g_AnySlave.encrypt = 0; // 暗号化無し
        memcpy(g_AnySlave.peer_addr, BroadcastAddress, sizeof(esp_now_peer_info::peer_addr));
        esp_now_add_peer(&g_AnySlave);
    }
    else
    {
        // TODO: 子機側
    }
}

void Scan(const char* pScanSsidPrefix) noexcept
{
    int discoveredCount = WiFi.scanNetworks();
    memset(g_Slaves, 0, sizeof(g_Slaves));
    g_SlaveCount = 0;

    if (discoveredCount == 0)
    {
        WiFi.scanDelete();
        return;
    }

    for (int i = 0; i < discoveredCount; i++)
    {
        String ssid = WiFi.SSID(i);
        int32_t rssi = WiFi.RSSI(i);
        String bssIdStr = WiFi.BSSIDstr(i);

        // DEBUG:
        LOG("%d: %s [%s] (%d)\n", i + 1, ssid.c_str(), bssIdStr.c_str(), rssi);

        if (ssid.indexOf(String(pScanSsidPrefix)) < 0)
        {
            continue;
        }

        g_Slaves[g_SlaveCount].channel = EspNowChannel;
        g_Slaves[g_SlaveCount].encrypt = 0; // 暗号化無し
        g_SlaveCount++;

        if (g_SlaveCount >= EspNowSlaveMax)
        {
            break;
        }
    }

    LOG("Registerd Slave Count : %d\n", g_SlaveCount);

    WiFi.scanDelete();
}

// TORIAEZU:

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

}

void loop()
{
    // TODO: DIPSW で分岐

    /*
    // TODO: スレーブの命名規則を正式に決める
    Scan("Slave");

    if (g_SlaveCount > 0)
    {
        uint8_t data[64] =
        {
            '7', 'S', 'E', 'G',
        };

        LOG("call esp_now_send()\n");
        esp_now_send(BroadcastAddress, data, sizeof(data));
    }

    LOG("Hello World!\n");
    delay(1000);
    */

    ////////////////////////////////////////////////////////////
    // TORIAEZU:
    ////////////////////////////////////////////////////////////
    
    int currentTick = millis();

    static int s_NextUpdateTick = 1000; // TORIAEZU:
    if (currentTick + 500 < s_NextUpdateTick)
    {
        return;
    }
    s_NextUpdateTick += 500;

    // TODO: 輝度更新と数字更新の頻度は独立させるべき

    // 輝度更新はあまり高頻度では行わない
    g_Building.SetBrightness(4);

    static bool s_ReverseMode = false;

    static int s_Number = 0;
    s_Number++;
    if (s_Number >= 10) {
        s_Number = 0;
    }

    if (s_ReverseMode)
    {
        g_Building.Fill();
    }
    else
    {
        g_Building.Clear();
    }
    uint8_t numberPattern = NumberSegmentTable[s_Number];
    for (int metaSeg = 0; metaSeg < 8; metaSeg++)
    {
        if (numberPattern & (1 << (7 - metaSeg))) {
            for (int y = 0; y < Building::DigitY; y++)
            {
                for (int x = 0; x < Building::DigitX; x++)
                {
                    uint8_t pattern = MetaSegmentTable[metaSeg][y][x];

                    if (s_ReverseMode)
                    {
                        g_Building.AndPattern(x, y, pattern);
                    }
                    else
                    {
                        g_Building.OrPattern(x, y, pattern);
                    }
                }
            }
        }
    }
    g_Building.Update();
}
