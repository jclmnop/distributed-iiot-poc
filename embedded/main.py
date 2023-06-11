import mqtt
from connect_wifi import connect_wifi
from time import time, sleep
from machine import Timer, Pin
# from temp_sensor import current_temp, get_temp_sensor
from temp_sensor import InternalTemp
from wifi_credentials import SSID, PSWD
import json


SENSOR_ID = "f3088463-5623-476f-a1b5-ecb49446a443"
SENSOR_TYPE = "temp"
TOPIC_BASE = "picow"
TOPIC_HEARTBEAT = bytes(f"heartbeat/{TOPIC_BASE}", "utf-8")
TOPIC_TEMP_OUT = bytes(f"{TOPIC_BASE}/{SENSOR_ID}/read", "utf-8")
TOPIC_TEMP_POLL = bytes(f"{TOPIC_BASE}/{SENSOR_ID}/poll", "utf-8")
DISCONNECT_TOPIC = bytes(f"{TOPIC_BASE}/{SENSOR_ID}/disconnect", "utf-8")
# 28:CD:C1:03:E2:9F
MAC_ADDR = [40, 205, 193, 3, 226, 159]

ip_addr = connect_wifi(SSID, PSWD)
client = mqtt.connect()
tim = Timer()
led: Pin = Pin("LED", Pin.OUT)
temp = InternalTemp()

def heartbeat_cb(timer):
    global client
    current_time = time()
    print(f"HEARTBEAT {current_time}")
    heartbeat_info = {
        "id": SENSOR_ID, 
        "alias": "temp_01",
        "poll_interval": 60000,
        "poll_topic": TOPIC_TEMP_POLL,
        "read_topic": TOPIC_TEMP_OUT,
        "disconnect_topic": DISCONNECT_TOPIC,
        "ip_addr": ip_addr,
        "mac_addr": MAC_ADDR,
        "location": "rp-pico-w"
    }
    client.publish(TOPIC_HEARTBEAT, json.dumps(heartbeat_info))


def sub_cb(topic, msg):
    global led
    global client
    led.high()

    print(f"RECEIVED\ntopic: {topic}\nmsg: {msg}\n")

    if topic == TOPIC_TEMP_POLL:
        print("TEMP_POLL")
        data = {
            "sensor_id": SENSOR_ID, 
            "sensor_type": SENSOR_TYPE, 
            "value": temp.get_temp(),
            "timestamp": time()
        }
        client.publish(TOPIC_TEMP_OUT, json.dumps(data))
    
    led.low()


client.set_callback(sub_cb)
client.subscribe(TOPIC_TEMP_POLL)
tim.init(period=30_000, mode=Timer.PERIODIC, callback=heartbeat_cb)

while True:
    client.wait_msg()
