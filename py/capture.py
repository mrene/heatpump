import broadlink, time, sys
# --type 0x5216 --host 192.168.1.235 --mac ec0bae9fe2ef

dev = broadlink.hello('192.168.1.235')
dev.auth()


dev.enter_learning()
while True:    
    try:
        data = dev.check_data()
    except broadlink.exceptions.StorageError:
        time.sleep(0.5)
        continue
        
    hexdata = ''.join(format(x, '02x') for x in bytearray(data))
    print(hexdata)
    sys.stdout.flush()
    dev.enter_learning()