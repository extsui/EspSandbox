#include <esp_now.h>
#include <WiFi.h>
#include "Util.h"

// Global copy of slave
#define NUMSLAVES 20
esp_now_peer_info_t slaves[NUMSLAVES] = {};
int SlaveCnt = 0;

#define CHANNEL 1
#define PRINTSCANRESULTS 1

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

// Scan for slaves in AP mode
void ScanForSlave()
{
    int8_t scanResults = WiFi.scanNetworks();
    //reset slaves
    memset(slaves, 0, sizeof(slaves));
    SlaveCnt = 0;
    LOG("\n");
    if (scanResults == 0)
    {
        LOG("No WiFi devices in AP Mode found\n");
    }
    else
    {
        LOG("Found %d devices \n", scanResults);
        for (int i = 0; i < scanResults; i++)
        {
            // Print SSID and RSSI for each device found
            String SSID = WiFi.SSID(i);
            int32_t RSSI = WiFi.RSSI(i);
            String BSSIDstr = WiFi.BSSIDstr(i);

            if (PRINTSCANRESULTS)
            {
                LOG("%d: %s [%s] (%d)\n", i + 1, SSID.c_str(), BSSIDstr.c_str(), RSSI);
            }
            delay(10);
            // Check if the current device starts with `Slave`
            if (SSID.indexOf("Slave") == 0)
            {
                // SSID of interest
                LOG("%d: %s [%s] (%d)\n", i + 1, SSID.c_str(), BSSIDstr.c_str(), RSSI);
                // Get BSSID => Mac Address of the Slave
                int mac[6];

                if (6 == sscanf(BSSIDstr.c_str(), "%x:%x:%x:%x:%x:%x", &mac[0], &mac[1], &mac[2], &mac[3], &mac[4], &mac[5]))
                {
                    for (int ii = 0; ii < 6; ii++)
                    {
                        slaves[SlaveCnt].peer_addr[ii] = (uint8_t) mac[ii];
                    }
                }
                slaves[SlaveCnt].channel = CHANNEL; // pick a channel
                slaves[SlaveCnt].encrypt = 0; // no encryption
                SlaveCnt++;
            }
        }
    }

    if (SlaveCnt > 0)
    {
        LOG("%d Slave(s) found, processing..\n", SlaveCnt);
    }
    else
    {
        LOG("No Slave Found, trying again.\n");
    }

    // clean up ram
    WiFi.scanDelete();
}

// Check if the slave is already paired with the master.
// If not, pair the slave with master
void manageSlave()
{
    if (SlaveCnt > 0)
    {
        for (int i = 0; i < SlaveCnt; i++)
        {
            LOG("Processing: ");
            for (int ii = 0; ii < 6; ii++)
            {
                LOG("%02X", (uint8_t) slaves[i].peer_addr[ii]);
                if (ii != 5)
                {
                    LOG(":");
                }
            }
            LOG(" Status: ");
            // check if the peer exists
            bool exists = esp_now_is_peer_exist(slaves[i].peer_addr);
            if (exists)
            {
                // Slave already paired.
                LOG("Already Paired\n");
            }
            else
            {
                // Slave not paired, attempt pair
                esp_err_t addStatus = esp_now_add_peer(&slaves[i]);
                if (addStatus == ESP_OK)
                {
                    // Pair success
                    LOG("Pair success\n");
                }
                else if (addStatus == ESP_ERR_ESPNOW_NOT_INIT)
                {
                    // How did we get so far!!
                    LOG("ESPNOW Not Init\n");
                }
                else if (addStatus == ESP_ERR_ESPNOW_ARG)
                {
                    LOG("Add Peer - Invalid Argument\n");
                }
                else if (addStatus == ESP_ERR_ESPNOW_FULL)
                {
                    LOG("Peer list full\n");
                }
                else if (addStatus == ESP_ERR_ESPNOW_NO_MEM)
                {
                    LOG("Out of memory\n");
                }
                else if (addStatus == ESP_ERR_ESPNOW_EXIST)
                {
                    LOG("Peer Exists\n");
                }
                else
                {
                    LOG("Not sure what happened\n");
                }
                delay(100);
            }
        }
    }
    else
    {
        // No slave found to process
        LOG("No Slave found to process\n");
    }
}

uint8_t data = 0;
// send data
void sendData()
{
    data++;
    for (int i = 0; i < SlaveCnt; i++)
    {
        const uint8_t *peer_addr = slaves[i].peer_addr;
        if (i == 0)
        {
            // print only for first slave
            LOG("Sending: %d\n", data);
        }
        esp_err_t result = esp_now_send(peer_addr, &data, sizeof(data));
        LOG("Send Status: ");
        if (result == ESP_OK)
        {
            LOG("Success\n");
        }
        else if (result == ESP_ERR_ESPNOW_NOT_INIT)
        {
            // How did we get so far!!
            LOG("ESPNOW not Init.\n");
        }
        else if (result == ESP_ERR_ESPNOW_ARG)
        {
            LOG("Invalid Argument\n");
        }
        else if (result == ESP_ERR_ESPNOW_INTERNAL)
        {
            LOG("Internal Error\n");
        }
        else if (result == ESP_ERR_ESPNOW_NO_MEM)
        {
            LOG("ESP_ERR_ESPNOW_NO_MEM\n");
        }
        else if (result == ESP_ERR_ESPNOW_NOT_FOUND)
        {
            LOG("Peer not found.\n");
        }
        else
        {
            LOG("Not sure what happened\n");
        }
        delay(100);
    }
}

// callback when data is sent from Master to Slave
void OnDataSent(const uint8_t *mac_addr, esp_now_send_status_t status)
{
    char macStr[18];
    snprintf(macStr, sizeof(macStr), "%02x:%02x:%02x:%02x:%02x:%02x",
             mac_addr[0], mac_addr[1], mac_addr[2], mac_addr[3], mac_addr[4], mac_addr[5]);
    LOG("Last Packet Sent to: %s\n", macStr);
    LOG("Last Packet Send Status: %s\n", (status == ESP_NOW_SEND_SUCCESS ? "Delivery Success" : "Delivery Fail"));
}

void setup()
{
    Serial.begin(115200);
    //Set device in STA mode to begin with
    WiFi.mode(WIFI_STA);
    LOG("ESPNow/Multi-Slave/Master Example\n");
    // This is the mac address of the Master in Station Mode
    LOG("STA MAC: %s\n", WiFi.macAddress());
    // Init ESPNow with a fallback logic
    InitESPNow();
    // Once ESPNow is successfully Init, we will register for Send CB to
    // get the status of Trasnmitted packet
    esp_now_register_send_cb(OnDataSent);
}

void loop()
{
    // In the loop we scan for slave
    ScanForSlave();
    // If Slave is found, it would be populate in `slave` variable
    // We will check if `slave` is defined and then we proceed further
    if (SlaveCnt > 0)
    {
        // check if slave channel is defined
        // `slave` is defined
        // Add slave as peer if it has not been added already
        manageSlave();
        // pair success or already paired
        // Send data to device
        sendData();
    }
    else
    {
        // No slave found to process
    }

    // wait for 3seconds to run the logic again
    delay(1000);
}
