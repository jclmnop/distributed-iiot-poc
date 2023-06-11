import network
import ubinascii
from time import sleep

def connect_wifi(ssid: str, password: str | None):
    wlan = network.WLAN(network.STA_IF)
    wlan.active(True)
    mac: str = str(wlan.config("mac")).replace('\\x', ':')[4:-1].upper()
    print(f"\nMAC: {mac}")
    wlan.connect(ssid, password)
    max_wait = 10
    while max_wait > 0:
        if wlan.status() < 0 or wlan.status() >= 3:
            break
        max_wait -= 1
        print('waiting for connection...')
        sleep(1)

    print(f"connection status: {wlan.status()}")

    if wlan.status() != 3:
        raise RuntimeError('wifi connection failed')
    else:
        print('connected')
        status = wlan.ifconfig()
        print(f"ip: {status[0]}\n")
    return status[0]