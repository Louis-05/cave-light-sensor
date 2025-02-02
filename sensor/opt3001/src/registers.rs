device_driver::create_device!(
    device_name: Opt3001LowLevel,
    dsl: {
        config {
            type RegisterAddressType = u8;
            type DefaultByteOrder = LE;
            type DefaultBitOrder = LSB0;
        }

        register Measurment{
            type Access = RO;

            const ADDRESS = 0x00;
            const SIZE_BITS = 16;

            fractional: uint = 0..=11,
            exponent: uint = 12..=15,
        },
        register Configuration{
            type Access = RW;

            const ADDRESS = 0x01;
            const SIZE_BITS = 16;
            fault_count: uint as enum FaultCount {
                One = 0b00,
                Two = 0b01,
                Four = 0b10,
                Eight = 0b11
            } = 0..=1,
            mask : bool = 2,
            polarity: bool = 3,
            latch: bool = 4,
            flag_low: RO bool = 5,
            flag_high: RO bool = 6,
            conversion_ready: RO bool = 7,
            overflow_flag: RO bool = 8,
            convertion_mode: uint as enum ConvertionMode {
                Shutdown = 0b00,
                SingleShot = 0b01,
                Continuous = catch_all,
            } = 9..=10,
            conversion_time: uint as enum ConvertionTime {
                Ms100 = 0,
                Ms800 = 1
            } = 11..=11,
            range_number : uint = 12..=15
        },
        register Lowlimit{
            type Access = RW;

            const ADDRESS = 0x02;
            const SIZE_BITS = 16;

            fractional: uint = 0..=11,
            exponent: uint = 12..=15,
        },
        register Highlimit{
            type Access = RW;

            const ADDRESS = 0x03;
            const SIZE_BITS = 16;

            fractional: uint = 0..=11,
            exponent: uint = 12..=15,
        },
        register ManufacturerID{
            type Access = RO;

            const ADDRESS = 0x7E;
            const SIZE_BITS = 16;

            id: uint = 0..=15,
        },
        register DeviceID{
            type Access = RO;

            const ADDRESS = 0x7F;
            const SIZE_BITS = 16;

            id: uint = 0..=15,
        },
    }
);
