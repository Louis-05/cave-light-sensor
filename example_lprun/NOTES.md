
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

https://www.st.com/resource/en/reference_manual/DM00108282-.pdf

https://www.st.com/resource/en/reference_manual/DM00108282-.pdf#page=150&zoom=100,89,238

https://www.st.com/resource/en/reference_manual/DM00108282-.pdf#page=156&zoom=100,165,121