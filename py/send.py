import broadlink, time, sys
# --type 0x5216 --host 192.168.1.235 --mac ec0bae9fe2ef

dev = broadlink.hello('192.168.1.235')
dev.auth()


for line in sys.stdin:
    data = bytearray.fromhex(line.rstrip())
    dev.send_data(data)
    print('Sent')
    