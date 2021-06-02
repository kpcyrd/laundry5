# laundry5

Shuffles your socks - rotating proxy frontend server.

    # start proxy server
    laundry5 -v -B 127.0.0.1:1337 -L ./pr0xies.txt
    # add new proxies to list
    echo 127.0.0.1:9050 | anewer ./pr0xies.txt
    # reload proxy list
    killall -HUP laundry5
    # send a request through a random proxy from list
    curl -vx socks5h://127.0.0.1:1337 https://icanhazip.com/

## License

GPLv3+
