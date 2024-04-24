import serial

serial_port = 'COM27'
baud_rate = 1000000
ser = serial.Serial(serial_port, baud_rate)

try:
    while True:
        serial_data = ser.readline().decode().strip()
        print(serial_data)
except KeyboardInterrupt:
    ser.close()
    