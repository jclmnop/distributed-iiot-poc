from umqtt.simple import MQTTClient

def connect():
    client = MQTTClient(
        client_id=b"picow_sensor1",
        server=b"pi4.local",
        port=1883,
        keepalive=7200,
    )

    client.connect()
    return client
