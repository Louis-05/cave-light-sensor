
```
sudo service udev stop && \
sudo service udev restart && \
sudo udevadm control --reload-rules && \
sudo service udev stop && \
sudo service udev start
```

rustup toolchain install nightly
rustup target add --toolchain nightly thumbv6m-none-eabi
cargo +nightly build
