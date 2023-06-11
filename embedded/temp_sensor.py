# import onewire, ds18x20, time
# from machine import Pin
import machine

class InternalTemp:
    """Read the internal temp sensor"""

    def __init__(self):
        """Init"""
        self.sensor_temp = machine.ADC(4)
        self.convert = 3.3 / 65535
        self.temperature = 0

    def get_temp(self):
        """Read the Internal Temp Sensor"""
        reading = self.sensor_temp.read_u16() * self.convert
        self.temperature = 27 - (reading - 0.706) / 0.001721
        return self.temperature

# Didn't manage to get this working and I'm running out of time so will have to 
# just use the temperature of the cpu
#
# def get_temp_sensor() -> Tuple[ds18x20.DS18X20, Any]:
#     SensorPin = Pin(26, Pin.IN)
#     sensor = ds18x20.DS18X20(onewire.OneWire(SensorPin))
#     roms = sensor.scan()
#     print(roms)
#     sensor.convert_temp()
#     time.sleep(2)
#     return (sensor, roms[0])

# def current_temp(sensor: Tuple[ds18x20.DS18X20, Any]) -> int:
#     return round(sensor[0].read_temp(sensor[1]))

