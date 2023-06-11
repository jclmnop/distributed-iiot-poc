# Distributed IIoT - Proof of Concept

[Pangea's API is utilised here](https://github.com/jclmnop/distributed-iiot-poc/tree/master/wasmcloud/actors/pangea-api)

This is a proof of concept for a distributed IIoT system, and utilises Pangea 
to maintain tamper proof audit logs of sensor readings and any other actions such 
as user logins etc.

The repo is split into three main parts.

## WasmCloud
There are three wasmCloud hosts running, one on a DigitalOcean Droplet in Singapore,
one on a Raspberry Pi in my house, and one on Cosmonic's managed infrastructure (which
is also what connects them all together).

The **pangea-api** actor is responsible for building and sending any requests to the
Pangea API. 

The **nats-polling-provider** is responsible for picking up heartbeats from sensors, 
and then polling them at specified intervals to get their readings. These readings
are then sent to the **sensor-reader** actor, which is responsible for processing
and sending them to the **paangea-api** actor.

The **http-gateway** actor is responsible for exposing the **pangea-api** actor's
functionality to the outside world via HTTP. It also handles requests for the UI
assets from web browsers, but I haven't implemented the **ui-actor** yet.

## Simulation
A simple rust program which is currently running on a Droplet in Singapore alongside
a wasmCloud host. It simulates about 40 sensors all polling at different intervals. 

Each sensor has a unique UUID, and this ID is used to create unique poll/read
topics for each sensor, so the nats-polling-provider on the Cosmonic managed 
wasmCloud host can pick up a sensor's heartbeat and then poll/read the correct
NATS subjects for that sensor.

## Embedded
Unfortunately this code is kind of redundant, because I was unable to get it to 
connect to the cluster on my own WiFi, despite getting it to work round my friend's
house. 

It uses MQTT instead of NATS, but that's only because I couldn't find a 
micropython or `no-std` Rust NATS client. If I had more time I probably would 
have just written my own `no-std` client.

Luckily, it's quite easy to integrate MQTT into a NATS server, and it behaves 
in pretty much the same way. 

In a real system, one or more sensors would be connected to a single microcontroller,
and the microcontrollers would all connect to each other to form a mesh, while also
connecting to the WiFi if they're in range of an access point. Their NATS messages
would be routed through the mesh via the shortest path to a simple computer such as 
a Raspberry Pi, which would be running a wasmCloud host. Each local machine (Raspberry Pi)
could either have all the necessary actors and providers running on it, or it could
just act as a gateway to a supercluster (or both, for extra redundancy). 

In a real system I'd also use some kind of local database to cache the sensor readings
if there's ever some extended downtime, and then upload them in bulk to the Pangea API.