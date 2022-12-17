This is a very quick and dirty implementation of an IR encoder/decoder for a Lennox Heat pump unit for use with a Broadlink RM4 Mini

## Usage

### generate ir code

```
heatpump set-state --power --mode Heat -t 24 --fan Auto | python ./py/send.py
```

### decode ir code

```
$ cat captures/off.ir | heatpump decode
Recv: a12347ffffeb 101000010010001101000111111111111111111111101011
Decode: Ok(ControlState { power: false, mode: Heat, temperature: Some(24), fan: Auto })
```


## References
- [Broadlink IR format converter (node.js)](https://github.com/haimkastner/broadlink-ir-converter)
- [python-broadlink's protocol description](https://github.com/mjg59/python-broadlink/blob/master/protocol.md)
- [lennox-ir python project](https://github.com/efficks/lennoxir) and its [golang port](https://github.com/efficks/golennoxir)

