#include <Arduino.h>
#include <Wire.h>
#include <stdint.h>
#include "Building.h"
#include "Tone.h"
#include "Util.h"
#include "Volume.h"

// TODO: 後で分離
#include <esp_now.h>
#include <WiFi.h>

namespace Pin {

constexpr int DipSwitch0 = D2;
constexpr int DipSwitch1 = D3;
constexpr int DipSwitch2 = D4;
constexpr int DipSwitch3 = D5;

constexpr int Buzzer = D6;

constexpr int LeftButton = D8;
constexpr int RightButton = D9;

constexpr int LeftVolume = A0;
constexpr int RightVolume = A1;

}

namespace {

Building g_Building;

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

Volume g_LeftVolume;
Volume g_RightVolume;

// RC フィルタ比率 (1~99)
constexpr int LowPassFilterRate = 80;
RcFilter g_LeftVolumeFilter(LowPassFilterRate);
RcFilter g_RightVolumeFilter(LowPassFilterRate);

constexpr uint32_t LeftVolumeLevel = 16; // 輝度調整向け
constexpr uint32_t RightVolumeLevel = 100;  // 範囲は適当

}

// ESP_NOW マスタ関連
namespace {

// TORIEAZU: 親機専用
void OnDataSendCompleteCallback(const uint8_t *pMac, esp_now_send_status_t status)
{
    char macStr[18];
    snprintf(macStr, sizeof(macStr), "%02x:%02x:%02x:%02x:%02x:%02x",
             pMac[0], pMac[1], pMac[2], pMac[3], pMac[4], pMac[5]);
    LOG("Last Packet Sent to: %s\n", macStr);
    LOG("Last Packet Send Status: %s\n", (status == ESP_NOW_SEND_SUCCESS ? "Delivery Success" : "Delivery Fail"));
}

// TORIAEZU: 子機専用
void OnDataReceiveCallback(const uint8_t *pMac, const uint8_t *data, int length)
{
    constexpr char magic[4] = { '7', 'S', 'E', 'G' };
    if (memcmp(data, magic, sizeof(magic)) != 0)
    {
        return;
    }

    const uint8_t* pattern = &data[4];

    char macStr[18];
    snprintf(macStr, sizeof(macStr), "%02x:%02x:%02x:%02x:%02x:%02x",
             pMac[0], pMac[1], pMac[2], pMac[3], pMac[4], pMac[5]);
    LOG("Last Packet Recv from: %s\n", macStr);

    g_Building.SetPatternAll(pattern, 36);

    // TODO: コールバック内で重い処理をしてよいのか要確認
    g_Building.Update();

    LOG("Last Packet Recv Data: \n");
    HexDump(data, length);
    LOG("\n");
}

}

constexpr int EspNowSlaveMax = 15;
esp_now_peer_info_t g_Slaves[EspNowSlaveMax] = {};
esp_now_peer_info_t g_AnySlave = {};
int g_SlaveCount = 0;

constexpr int EspNowChannel = 1;

constexpr uint8_t BroadcastAddress[] = { 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF };

TwoWire& g_Wire = Wire;

void InitializeByStandAloneMode() noexcept
{
    g_Wire.begin(D10, D7, 400000);
    g_Building.Initialize(&g_Wire);
    g_Building.Clear();
    g_Building.Update();
}

void InitializeEspNow() noexcept
{
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

    g_LeftVolume.Initialize(Pin::LeftVolume, &g_LeftVolumeFilter, LeftVolumeLevel);
    g_RightVolume.Initialize(Pin::RightVolume, &g_RightVolumeFilter, RightVolumeLevel);
    LOG("Volume: Initialized.\n");

    LOG("OwnAddress: 0x%02x\n", g_OwnAddress);

    // TODO: 以降は後で分離
    if (g_OwnAddress == 0)
    {
        WiFi.mode(WIFI_STA);
        LOG("Master MAC: %s\n", WiFi.macAddress().c_str());

        InitializeEspNow();

        esp_now_register_send_cb(OnDataSendCompleteCallback);

        // ブロードキャストする場合でも peer として登録しておく必要がある
        g_AnySlave.channel = EspNowChannel;
        g_AnySlave.encrypt = 0; // 暗号化無し
        memcpy(g_AnySlave.peer_addr, BroadcastAddress, sizeof(esp_now_peer_info::peer_addr));
        esp_now_add_peer(&g_AnySlave);
        
        Scan("extsui-Fountain");
    }
    else
    {
        // TODO: 子機側
        WiFi.mode(WIFI_AP);
        LOG("Slave MAC: %s\n", WiFi.softAPmacAddress().c_str());

        char ssid[64];
        const char* password = "extsui-Fountain";
        sprintf(ssid, "extsui-Fountain-%02x:%s", g_OwnAddress, WiFi.macAddress().c_str());
        bool result = WiFi.softAP(ssid, password, EspNowChannel, 0);
        if (!result)
        {
            LOG("AP Config failed.\n");
        }
        LOG("SSID: [%s]\n", ssid);

        InitializeEspNow();
        esp_now_register_recv_cb(OnDataReceiveCallback);
    }
}

void loop()
{
    // TODO: DIPSW で分岐

    // 子機は受信を待つだけ
    if (g_OwnAddress != 0)
    {
        return;
    }
    
    int currentTick = millis();

    g_LeftVolume.Update();
    g_RightVolume.Update();

    static int s_NextUpdateTick = 1000; // TORIAEZU:
    constexpr int UpdateIntervalBaseMilliSeconds = 50;
    static int s_UpdateInterval = UpdateIntervalBaseMilliSeconds;
    if (currentTick + s_UpdateInterval < s_NextUpdateTick)
    {
        return;
    }
    s_UpdateInterval = g_RightVolume.GetValue() + UpdateIntervalBaseMilliSeconds;
    s_NextUpdateTick += s_UpdateInterval;

    // TODO: 輝度更新と数字更新の頻度は独立させるべき

    // 輝度更新はあまり高頻度では行わない
    uint32_t brightness = g_LeftVolume.GetLevel();
    g_Building.SetBrightness(brightness);
    //LOG("Left: %d\n", g_LeftVolume.GetValue());

    static bool s_ReverseMode = false;
    static int s_Number = 0;
    s_Number++;
    if (s_Number >= 10) {
        s_Number = 0;
    }

    g_Building.Clear();
    g_Building.SetMetaNumberPattern(s_Number);
    if (s_ReverseMode)
    {
        g_Building.Reverse();
    }

    // TODO: パケット構造を定義すべし (サンプリングナンバー, 輝度)
    // 子機にブロードキャスト送信
    uint8_t data[64] =
    {
        '7', 'S', 'E', 'G',
    };
    g_Building.GetPatternAll(&data[4], 36);
    esp_now_send(BroadcastAddress, data, sizeof(data));

    g_Building.Update();
}
