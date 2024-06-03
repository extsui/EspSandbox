#include <esp_now.h>
#include <WiFi.h>
#include "Util.h"

namespace {

void HexDump(const uint8_t data[], size_t length) noexcept
{
    for (int i = 0; i < length; i++)
    {
        LOG("%02x ", data[i]);
        // 16 個表示毎に改行、ただし先頭行と最終データが丁度 16 の倍数の場合は省略
        if (((i + 1) % 16 == 0) && (i > 0) && ((i + 1) != length))
        {
            LOG("\n");
        }
    }
    LOG("\n");
}

}

#define CHANNEL 1

void OnDataRecv(const uint8_t *mac_addr, const uint8_t *data, int data_len);

// Init ESP Now with fallback
void InitESPNow()
{
    WiFi.disconnect();
    if (esp_now_init() == ESP_OK)
    {
        LOG("ESPNow Init Success\n");
    }
    else
    {
        LOG("ESPNow Init Failed\n");
        // Retry InitESPNow, add a counte and then restart?
        // InitESPNow();
        // or Simply Restart
        ESP.restart();
    }
}

// config AP SSID
void configDeviceAP()
{
    String Prefix = "Slave:";
    String Mac = WiFi.macAddress();
    String SSID = Prefix + Mac;
    String Password = "123456789";
    bool result = WiFi.softAP(SSID.c_str(), Password.c_str(), CHANNEL, 0);
    if (!result)
    {
        LOG("AP Config failed.\n");
    }
    else
    {
        LOG("AP Config Success. Broadcasting with AP: %s\n", SSID.c_str());
    }
}

void setup()
{
    Serial.begin(115200);

    // USB-CDC のせいか起動直後にログを大量に出しても PC 側に表示されない
    // 適当なディレイを入れると安定するようになったので暫定対処
    delay(1000);

    LOG("ESPNow/Basic/Slave Example\n");
    //Set device in AP mode to begin with
    WiFi.mode(WIFI_AP);
    // configure device AP mode
    configDeviceAP();
    // This is the mac address of the Slave in AP Mode
    LOG("AP MAC: %s\n", WiFi.softAPmacAddress().c_str());
    // Init ESPNow with a fallback logic
    InitESPNow();
    // Once ESPNow is successfully Init, we will register for recv CB to
    // get recv packer info.
    esp_now_register_recv_cb(OnDataRecv);
}

// callback when data is recv from Master
void OnDataRecv(const uint8_t *mac_addr, const uint8_t *data, int data_len)
{
    char macStr[18];
    snprintf(macStr, sizeof(macStr), "%02x:%02x:%02x:%02x:%02x:%02x",
             mac_addr[0], mac_addr[1], mac_addr[2], mac_addr[3], mac_addr[4], mac_addr[5]);
    LOG("Last Packet Recv from: %s\n", macStr);
    LOG("Last Packet Recv Data: \n");
    HexDump(data, data_len);
    LOG("\n");
}

void loop()
{
    LOG("alive\n");
    delay(1000);
}